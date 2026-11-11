//! The AI's runs over the whole library (Kotlin `AiBatches`): classify every document without a classification,
//! or tag every document without tags, one after the other in the AI lane until the user cancels. A run stops at
//! once when the AI server does not answer. Every suggestion waits under Open tasks as soon as it arrives.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use dms_application::classification::{ClassificationProgress, SuggestOutcome};
use dms_application::common::Progress;
use dms_application::port::ai::ClassificationFailure;
use dms_domain::classification::ClassificationSuggestion;
use dms_domain::document::DocumentId;

use crate::dto::IpcError;
use crate::session::reporting::busy;
use crate::session::{Lane, Operation, OperationProgress, Session, UiMessage, lock};

/// Which run: its label and progress step, and the message once it is done.
#[derive(Debug, Clone, Copy)]
pub struct AiRun {
    pub step: &'static str,
    done: fn(usize) -> String,
}

pub const CLASSIFY_ALL: AiRun = AiRun {
    step: "Classifying with AI",
    done: |count| format!("The AI classified {count} documents; review them under Open tasks"),
};

pub const TAG_ALL: AiRun = AiRun {
    step: "Tagging with AI",
    done: |count| format!("The AI suggested tags for {count} documents; review them under Open tasks"),
};

/// Starts `run`: `batch` calls the use case with the progress hook; `name_of` names a document in progress and log.
///
/// # Errors
/// `busy` while another AI run is going.
pub fn start<B, N>(session: &Arc<Session>, run: AiRun, batch: B, name_of: N) -> Result<(), IpcError>
where
    B: FnOnce(&dyn Progress<ClassificationProgress>) -> Option<ClassificationProgress> + Send + 'static,
    N: Fn(&DocumentId) -> String + Send + Sync + 'static,
{
    let started = session.perform(run.step, Lane::Ai, true, move |operation| {
        operation.progress(OperationProgress::new(run.step, 0, None));
        let hook = BatchProgress {
            operation,
            step: run.step,
            name_of,
            suggested: AtomicUsize::new(0),
            stopped: Mutex::new(None),
        };
        batch(&hook);
        if operation.is_cancelled() {
            return Ok(());
        }
        let message = match *lock(&hook.stopped) {
            Some(reason) => UiMessage::error(failure(reason)),
            None => UiMessage::info((run.done)(hook.suggested.load(Ordering::SeqCst))),
        };
        operation.session().report(message);
        Ok(())
    });
    if started { Ok(()) } else { Err(busy(session, Lane::Ai, run.step)) }
}

struct BatchProgress<'o, N> {
    operation: &'o Operation,
    step: &'static str,
    name_of: N,
    suggested: AtomicUsize,
    stopped: Mutex<Option<ClassificationFailure>>,
}

impl<N: Fn(&DocumentId) -> String + Sync> Progress<ClassificationProgress> for BatchProgress<'_, N> {
    fn report(&self, progress: &ClassificationProgress) {
        let name = (self.name_of)(&progress.document_id);
        let session = self.operation.session();
        self.operation
            .progress(OperationProgress::new(self.step, progress.done, Some(progress.total)).with_detail(name.clone()));
        match &progress.outcome {
            SuggestOutcome::Suggested(suggestion) if has_values(suggestion) => {
                self.suggested.fetch_add(1, Ordering::SeqCst);
                session.info(format!("AI suggestion for {name}"));
                session.changed();
            }
            SuggestOutcome::Suggested(_) => session.info(format!("The AI found nothing new for {name}")),
            outcome => session.error(suggest_message(outcome, &name).text),
        }
        if let SuggestOutcome::Failed { reason: ClassificationFailure::ProviderUnavailable, .. } = &progress.outcome {
            *lock(&self.stopped) = Some(ClassificationFailure::ProviderUnavailable);
        }
    }

    fn is_cancelled(&self) -> bool {
        self.operation.is_cancelled() || lock(&self.stopped).is_some()
    }
}

fn has_values(suggestion: &ClassificationSuggestion) -> bool {
    !suggestion.tags.is_empty()
        || suggestion.title.is_some()
        || suggestion.category.is_some()
        || suggestion.document_date.is_some()
        || suggestion.correspondent.is_some()
        || suggestion.file_name.is_some()
}

// shortcut: Kotlin `AiMessages` (review area); move to `crate::messages` once the document area needs it too.
pub fn suggest_message(outcome: &SuggestOutcome, name: &str) -> UiMessage {
    match outcome {
        SuggestOutcome::Suggested(_) => UiMessage::info(format!("The AI suggested a classification for {name}")),
        SuggestOutcome::LibraryNotOpen => UiMessage::error("No library is open"),
        SuggestOutcome::NotFound => UiMessage::error(format!("{name} is no longer in the library")),
        SuggestOutcome::NotReadable { detail } => UiMessage::error(format!(
            "{name} could not be read for the AI{}",
            detail.as_ref().map_or_else(String::new, |detail| format!(": {detail}"))
        )),
        SuggestOutcome::Failed { reason, .. } => UiMessage::error(failure(*reason)),
    }
}

pub fn failure(reason: ClassificationFailure) -> &'static str {
    match reason {
        ClassificationFailure::ProviderUnavailable => {
            "The AI does not answer. Start LM Studio's server, or for Anthropic set ANTHROPIC_API_KEY; check Settings › AI"
        }
        ClassificationFailure::ModelCannotReadImages => {
            "The model cannot read images. Load a vision model, such as Gemma 3 or Qwen2.5-VL"
        }
        ClassificationFailure::InvalidResponse => "The AI's answer could not be understood; try again",
        ClassificationFailure::Timeout => "The AI took too long; try again, or raise the timeout in Settings › AI",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::mpsc::{Receiver, Sender, channel};
    use std::time::Duration;

    use super::*;
    use crate::session::{LogEntry, SessionEvents, SessionState};

    struct Recorder {
        messages: Mutex<Vec<UiMessage>>,
        log: Mutex<Vec<String>>,
        idle: Mutex<Sender<()>>,
    }

    impl SessionEvents for Recorder {
        fn state(&self, state: &SessionState) {
            if state.operations.is_empty() {
                let _ = lock(&self.idle).send(());
            }
        }
        fn progress(&self, _: &HashMap<Lane, OperationProgress>) {}
        fn log(&self, entry: &LogEntry) {
            lock(&self.log).push(entry.text.clone());
        }
        fn message(&self, message: &UiMessage) {
            lock(&self.messages).push(message.clone());
        }
    }

    fn session() -> (Arc<Session>, Arc<Recorder>, Receiver<()>) {
        let (idle, finished) = channel();
        let recorder = Arc::new(Recorder {
            messages: Mutex::new(Vec::new()),
            log: Mutex::new(Vec::new()),
            idle: Mutex::new(idle),
        });
        (Session::new(recorder.clone()), recorder, finished)
    }

    fn id(value: &str) -> DocumentId {
        DocumentId::new(value).expect("id")
    }

    fn suggested(document: &str, with_title: bool) -> SuggestOutcome {
        let mut suggestion = ClassificationSuggestion::empty(id(document), "model");
        if with_title {
            suggestion.title = Some("Invoice".to_owned());
        }
        SuggestOutcome::Suggested(suggestion)
    }

    fn step(done: usize, document: &str, outcome: SuggestOutcome) -> ClassificationProgress {
        ClassificationProgress { done, total: 3, document_id: id(document), outcome }
    }

    fn wait(finished: &Receiver<()>) {
        finished.recv_timeout(Duration::from_secs(5)).expect("the run finishes");
    }

    #[test]
    fn counts_only_suggestions_with_values() {
        let (session, recorder, finished) = session();
        let batch = |progress: &dyn Progress<ClassificationProgress>| {
            progress.report(&step(1, "d1", suggested("d1", true)));
            progress.report(&step(2, "d2", suggested("d2", false)));
            progress.report(&step(3, "d3", SuggestOutcome::NotFound));
            None
        };
        start(&session, TAG_ALL, batch, |document| format!("{document}.pdf")).expect("started");
        wait(&finished);
        assert_eq!(
            *lock(&recorder.messages),
            vec![UiMessage::info("The AI suggested tags for 1 documents; review them under Open tasks")]
        );
        let log = lock(&recorder.log).clone();
        assert!(log.contains(&"AI suggestion for d1.pdf".to_owned()));
        assert!(log.contains(&"The AI found nothing new for d2.pdf".to_owned()));
        assert!(log.contains(&"d3.pdf is no longer in the library".to_owned()));
    }

    #[test]
    fn stops_at_once_when_the_server_does_not_answer() {
        let (session, recorder, finished) = session();
        let calls = Arc::new(AtomicUsize::new(0));
        let counted = Arc::clone(&calls);
        let batch = move |progress: &dyn Progress<ClassificationProgress>| {
            for (index, document) in ["d1", "d2", "d3"].iter().enumerate() {
                if progress.is_cancelled() {
                    break;
                }
                counted.fetch_add(1, Ordering::SeqCst);
                let outcome =
                    SuggestOutcome::Failed { reason: ClassificationFailure::ProviderUnavailable, detail: None };
                progress.report(&step(index + 1, document, outcome));
            }
            None
        };
        start(&session, CLASSIFY_ALL, batch, ToString::to_string).expect("started");
        wait(&finished);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            *lock(&recorder.messages),
            vec![UiMessage::error(failure(ClassificationFailure::ProviderUnavailable))]
        );
    }
}
