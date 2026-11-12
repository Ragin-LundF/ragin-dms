use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::{Lane, LogEntry, OperationProgress, Session, SessionEvents, SessionLog, SessionState, UiMessage};

/// Records what the session pushed; `finished` gets the state whenever no operation runs anymore.
struct Recorder {
    messages: Mutex<Vec<UiMessage>>,
    idle: Mutex<Sender<SessionState>>,
}

impl SessionEvents for Recorder {
    fn state(&self, state: &SessionState) {
        if state.operations.is_empty() {
            let _ = self.idle.lock().expect("lock").send(state.clone());
        }
    }
    fn progress(&self, _: &HashMap<Lane, OperationProgress>) {}
    fn log(&self, _: &LogEntry) {}
    fn message(&self, message: &UiMessage) {
        self.messages.lock().expect("lock").push(message.clone());
    }
}

fn session() -> (Arc<Session>, Arc<Recorder>, Receiver<SessionState>) {
    let (idle, finished) = channel();
    let recorder = Arc::new(Recorder { messages: Mutex::new(Vec::new()), idle: Mutex::new(idle) });
    (Session::new(recorder.clone()), recorder, finished)
}

fn wait(finished: &Receiver<SessionState>) -> SessionState {
    finished.recv_timeout(Duration::from_secs(5)).expect("the operation finishes")
}

#[test]
fn a_busy_lane_refuses_a_second_operation_but_another_lane_runs() {
    let (session, _, finished) = session();
    let (release, gate) = channel::<()>();
    assert!(
        session.perform("Scanning", Lane::Library, false, move |_| { gate.recv().map_err(|error| error.to_string()) })
    );
    assert!(!session.perform("Syncing", Lane::Library, false, |_| Ok(())));
    assert_eq!(session.state().busy_in(Lane::Library), Some("Scanning"));
    assert!(session.perform("Analyzing", Lane::Ai, false, |_| Ok(())));
    release.send(()).expect("release");
    let state = loop {
        let state = wait(&finished);
        if state.revision == 2 {
            break state;
        }
    };
    assert_eq!(state.busy_in(Lane::Library), None);
}

#[test]
fn a_failure_is_reported_with_the_label() {
    let (session, recorder, finished) = session();
    session.perform("Merging all folders", Lane::Library, false, |_| Err("disk full".to_owned()));
    wait(&finished);
    let messages = recorder.messages.lock().expect("lock").clone();
    assert_eq!(messages, vec![UiMessage::error("Merging all folders failed: disk full")]);
}

#[test]
fn cancel_stops_only_a_cancellable_operation() {
    let (session, recorder, finished) = session();
    let (started, running) = channel::<()>();
    session.perform("Scanning", Lane::Library, true, move |operation| {
        started.send(()).map_err(|error| error.to_string())?;
        while !operation.is_cancelled() {
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(())
    });
    running.recv().expect("started");
    session.cancel(Lane::Library);
    wait(&finished);
    let messages = recorder.messages.lock().expect("lock").clone();
    assert_eq!(messages, vec![UiMessage::info("Scanning: cancelled")]);
}

#[test]
fn for_each_with_progress_counts_successes_and_stops_when_cancelled() {
    let (session, _, finished) = session();
    let (counted, result) = channel();
    session.perform("Adding", Lane::Library, true, move |operation| {
        let items = [1, 2, 3, 4];
        let succeeded = operation.for_each_with_progress("Adding new files", &items, ToString::to_string, |item| {
            if *item == 3 {
                operation.session().cancel(Lane::Library);
            }
            item % 2 == 1
        });
        counted.send(succeeded).map_err(|error| error.to_string())
    });
    wait(&finished);
    assert_eq!(result.recv().expect("count"), 2);
}

#[test]
fn the_log_keeps_only_the_newest_lines() {
    let log = SessionLog::default();
    for index in 0..=SessionLog::CAPACITY {
        log.info(format!("line {index}"));
    }
    let entries = log.entries();
    assert_eq!(entries.len(), SessionLog::CAPACITY);
    assert_eq!(entries.first().map(|entry| entry.text.as_str()), Some("line 1"));
}

#[test]
fn cancel_frees_the_lane_while_a_blocking_call_still_runs() {
    let (session, _recorder, finished) = session();
    let (started, running) = channel::<()>();
    let (release, blocked) = channel::<()>();
    let (answered, answer) = channel::<Option<u32>>();
    session.perform("Asking the library", Lane::Ai, true, move |operation| {
        let result = operation.run_until_cancelled(move || {
            let _ = started.send(());
            // Stands for a request to the AI server that cannot be interrupted.
            let _ = blocked.recv();
            42
        });
        answered.send(result).map_err(|error| error.to_string())
    });
    running.recv().expect("started");
    session.cancel(Lane::Ai);
    wait(&finished);
    assert_eq!(answer.recv().expect("answer"), None);
    assert!(session.state().busy_in(Lane::Ai).is_none());
    drop(release);
}

#[test]
fn a_yielding_operation_stops_for_another_operation_of_its_lane_and_knows_it_was_preempted() {
    let (session, recorder, _finished) = session();
    let (preempted_tx, preempted_rx) = channel::<bool>();
    let (ran_tx, ran_rx) = channel::<()>();
    assert!(session.perform_yielding("Checking the archive", Lane::Library, move |operation| {
        while !operation.is_cancelled() {
            std::thread::yield_now();
        }
        let _ = preempted_tx.send(operation.is_preempted());
        Ok(())
    }));

    assert!(session.perform("Saving the catalog", Lane::Library, false, move |_| {
        let _ = ran_tx.send(());
        Ok(())
    }));

    assert!(preempted_rx.recv_timeout(Duration::from_secs(5)).expect("the check stopped"));
    ran_rx.recv_timeout(Duration::from_secs(5)).expect("the action ran");
    assert_eq!(recorder.messages.lock().expect("lock").len(), 0, "no 'cancelled' message for a preemption");
}

#[test]
fn an_ordinary_operation_is_never_preempted() {
    let (session, _, finished) = session();
    let (release, gate) = channel::<()>();
    assert!(session.perform("Scanning", Lane::Library, true, move |_| gate.recv().map_err(|error| error.to_string())));
    assert!(!session.perform("Saving the catalog", Lane::Library, false, |_| Ok(())));
    release.send(()).expect("release");
    wait(&finished);
}
