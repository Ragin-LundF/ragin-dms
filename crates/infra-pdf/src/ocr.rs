use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::error::PdfError;
use crate::files::{self, FileFacts};

const PROGRAM: &str = "ocrmypdf";
const DEFAULT_TIMEOUT: Duration = Duration::from_mins(10);
const POLL: Duration = Duration::from_millis(100);

/// Runs an external program; tests replace it, so they never need the program installed.
pub trait CommandRunner: Send + Sync {
    /// The exit code of [`command`], or `None` if it could not be started or did not finish in time.
    fn run(&self, command: &[OsString]) -> Option<i32>;
}

/// Runs a program with a time limit; its output is discarded.
pub struct ProcessRunner {
    pub timeout: Duration,
}

impl Default for ProcessRunner {
    fn default() -> Self {
        ProcessRunner { timeout: DEFAULT_TIMEOUT }
    }
}

impl CommandRunner for ProcessRunner {
    fn run(&self, command: &[OsString]) -> Option<i32> {
        let (program, arguments) = command.split_first()?;
        let mut child = Command::new(program)
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        let started = Instant::now();
        loop {
            if let Some(status) = child.try_wait().ok()? {
                return status.code();
            }
            if started.elapsed() >= self.timeout {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            thread::sleep(POLL);
        }
    }
}

/// "Precise OCR" with `ocrmypdf`, if it is installed and on the PATH; nothing is bundled. It writes a
/// searchable copy to a temporary file next to the target, which is then moved into place.
pub struct OcrMyPdf {
    runner: Box<dyn CommandRunner>,
    program: OsString,
}

impl Default for OcrMyPdf {
    fn default() -> Self {
        OcrMyPdf::new(Box::new(ProcessRunner::default()), PROGRAM)
    }
}

impl OcrMyPdf {
    #[must_use]
    pub fn new(runner: Box<dyn CommandRunner>, program: impl Into<OsString>) -> Self {
        OcrMyPdf { runner, program: program.into() }
    }

    /// Whether the program is installed and answers to `--version`.
    #[must_use]
    pub fn available(&self) -> bool {
        self.runner.run(&[self.program.clone(), "--version".into()]) == Some(0)
    }

    /// A searchable copy of [`source`] at [`target`], which must not exist yet (`ocrmypdf --skip-text
    /// --quiet`, so pages that already have text stay as they are).
    ///
    /// # Errors
    /// [`PdfError::NotFound`] for a missing source, [`PdfError::AlreadyExists`] if [`target`] exists,
    /// [`PdfError::ProgramFailed`] if the program fails or writes nothing.
    pub fn ocr(&self, source: &Path, target: &Path) -> Result<FileFacts, PdfError> {
        if !source.is_file() {
            return Err(PdfError::NotFound);
        }
        let program = self.program.to_string_lossy().into_owned();
        files::write_new(target, |temp| {
            let command = [self.program.clone(), "--skip-text".into(), "--quiet".into(), source.into(), temp.into()];
            let exit = self.runner.run(&command);
            if exit == Some(0) && temp.is_file() {
                Ok(())
            } else {
                // Kotlin wrote the exit value as is: "1", or "null" when the program was killed.
                let exit = exit.map_or_else(|| "null".to_owned(), |code| code.to_string());
                Err(PdfError::ProgramFailed(format!("{program} ended with {exit}")))
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::{Arc, Mutex};

    use super::*;

    struct Recorded {
        commands: Arc<Mutex<Vec<Vec<OsString>>>>,
        answer: fn(&[OsString]) -> Option<i32>,
    }

    impl CommandRunner for Recorded {
        fn run(&self, command: &[OsString]) -> Option<i32> {
            self.commands.lock().expect("lock").push(command.to_vec());
            (self.answer)(command)
        }
    }

    fn ocr_with(answer: fn(&[OsString]) -> Option<i32>) -> (OcrMyPdf, Arc<Mutex<Vec<Vec<OsString>>>>) {
        let commands = Arc::new(Mutex::new(Vec::new()));
        let runner = Recorded { commands: Arc::clone(&commands), answer };
        (OcrMyPdf::new(Box::new(runner), PROGRAM), commands)
    }

    #[test]
    fn the_program_is_available_when_it_answers_to_version() {
        assert!(ocr_with(|_| Some(0)).0.available());
        assert!(!ocr_with(|_| None).0.available());
        assert!(!ocr_with(|_| Some(1)).0.available());
    }

    #[test]
    fn the_searchable_copy_is_moved_to_the_target() {
        let root = tempfile::tempdir().expect("temp dir");
        fs::write(root.path().join("scan.pdf"), "%PDF scan").expect("write");
        let (ocr, commands) = ocr_with(|command| {
            fs::copy(&command[command.len() - 2], &command[command.len() - 1]).ok()?;
            Some(0)
        });

        let written = ocr.ocr(&root.path().join("scan.pdf"), &root.path().join("ocr.pdf"));

        assert!(written.is_ok(), "{written:?}");
        assert_eq!(fs::read_to_string(root.path().join("ocr.pdf")).expect("read"), "%PDF scan");
        let command = commands.lock().expect("lock")[0].clone();
        assert_eq!(command[..3], [OsString::from("ocrmypdf"), "--skip-text".into(), "--quiet".into()]);
    }

    #[test]
    fn a_failing_program_leaves_nothing_behind() {
        let root = tempfile::tempdir().expect("temp dir");
        fs::write(root.path().join("scan.pdf"), "%PDF scan").expect("write");
        let (ocr, _) = ocr_with(|command| {
            fs::write(&command[command.len() - 1], "half").ok()?;
            Some(2)
        });

        let written = ocr.ocr(&root.path().join("scan.pdf"), &root.path().join("ocr.pdf"));

        assert!(
            matches!(&written, Err(PdfError::ProgramFailed(detail)) if detail == "ocrmypdf ended with 2"),
            "{written:?}"
        );
        let names: Vec<_> = fs::read_dir(root.path()).expect("list").flatten().map(|e| e.file_name()).collect();
        assert_eq!(names, vec![OsString::from("scan.pdf")]);
    }

    #[test]
    fn a_killed_program_reads_null_like_kotlin() {
        let root = tempfile::tempdir().expect("temp dir");
        fs::write(root.path().join("scan.pdf"), "%PDF scan").expect("write");
        let (ocr, _) = ocr_with(|_| None);

        let written = ocr.ocr(&root.path().join("scan.pdf"), &root.path().join("ocr.pdf"));

        assert!(
            matches!(&written, Err(PdfError::ProgramFailed(detail)) if detail == "ocrmypdf ended with null"),
            "{written:?}"
        );
    }

    #[test]
    fn a_missing_program_is_reported_by_the_process_runner() {
        let runner = ProcessRunner::default();

        assert_eq!(runner.run(&["dms-no-such-program-4711".into()]), None);
    }
}
