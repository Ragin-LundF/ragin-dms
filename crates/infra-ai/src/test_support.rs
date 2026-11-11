//! A stub HTTP server on localhost and synthetic requests; tests never talk to a real AI server.
#![allow(clippy::expect_used)]

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use chrono::NaiveDate;
use dms_application::port::ai::{ClassificationRequest, DocumentContent, PageImage};
use dms_application::port::state::AiSettings;
use dms_domain::document::DocumentId;
use dms_domain::field::{CurrencyCode, FieldDefinition};
use dms_domain::library::RelativePath;
use dms_domain::outline::{FolderOutline, OutlineFolder};
use dms_domain::taxonomy::{Category, CategoryId, TagName};
use indexmap::IndexSet;
use serde_json::Value;

use crate::AiAdapters;
use crate::client::ChatTransport;

/// One request the stub received; header names are lower case.
#[derive(Debug, Clone)]
pub(crate) struct Received {
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Received {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(key, _)| key == name).map(|(_, value)| value.as_str())
    }

    pub fn json(&self) -> Value {
        serde_json::from_str(&self.body).expect("the request body is JSON")
    }
}

/// What the stub answers.
pub(crate) enum Reply {
    Status(u16, String),
    /// Waits this long before answering 200 with the body, to run into the client's timeout.
    Late(Duration, String),
}

pub(crate) fn ok(body: impl Into<String>) -> Reply {
    Reply::Status(200, body.into())
}

/// A chat completion whose message content is `content`.
pub(crate) fn chat_answer(content: &str) -> String {
    serde_json::json!({"choices": [{"message": {"role": "assistant", "content": content}}]}).to_string()
}

type Handler = Arc<dyn Fn(&Received) -> Reply + Send + Sync>;

/// A stub server on 127.0.0.1. Local requests go to `/v1/...`, Anthropic requests to `/anthropic/v1/...`.
pub(crate) struct StubServer {
    port: u16,
    received: Arc<Mutex<Vec<Received>>>,
}

impl StubServer {
    pub fn start(handler: impl Fn(&Received) -> Reply + Send + Sync + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("address").port();
        let received = Arc::new(Mutex::new(Vec::new()));
        let handler: Handler = Arc::new(handler);
        let log = Arc::clone(&received);
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let handler = Arc::clone(&handler);
                let log = Arc::clone(&log);
                thread::spawn(move || serve(stream, &handler, &log));
            }
        });
        Self { port, received }
    }

    pub fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}/v1", self.port)
    }

    pub fn anthropic_base_url(&self) -> String {
        format!("http://127.0.0.1:{}/anthropic/v1", self.port)
    }

    pub fn received(&self) -> Vec<Received> {
        self.received.lock().expect("lock").clone()
    }

    pub fn paths(&self) -> Vec<String> {
        self.received().into_iter().map(|request| request.path).collect()
    }

    /// The bodies of the chat requests, local or Anthropic.
    pub fn chat_bodies(&self) -> Vec<Value> {
        self.received()
            .iter()
            .filter(|request| request.path.ends_with("/chat/completions") || request.path.ends_with("/messages"))
            .map(Received::json)
            .collect()
    }

    /// Adapters on this stub with `settings`, whose base URL points here, and `key` as the Anthropic key.
    pub fn adapters(&self, settings: AiSettings, key: Option<&str>) -> AiAdapters {
        adapters_with(AiSettings { base_url: self.base_url(), ..settings }, key, &self.anthropic_base_url())
    }
}

/// Adapters with `settings` as they are and `key` as the Anthropic key, sent to `anthropic_base`.
pub(crate) fn adapters_with(settings: AiSettings, key: Option<&str>, anthropic_base: &str) -> AiAdapters {
    let key = key.map(str::to_owned);
    AiAdapters::of(ChatTransport::with_anthropic_base(
        Arc::new(move || settings.clone()),
        Arc::new(move || key.clone()),
        anthropic_base,
    ))
}

/// The base URL of a port nobody listens on.
pub(crate) fn unreachable_base_url() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("address").port();
    drop(listener);
    format!("http://127.0.0.1:{port}/v1")
}

fn serve(stream: TcpStream, handler: &Handler, log: &Mutex<Vec<Received>>) {
    let mut reader = BufReader::new(stream.try_clone().expect("clone stream"));
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }
    let path = request_line.split_whitespace().nth(1).unwrap_or_default().to_owned();
    let mut headers = Vec::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() || line.trim().is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_lowercase(), value.trim().to_owned()));
        }
    }
    let length = headers.iter().find(|(name, _)| name == "content-length").and_then(|(_, value)| value.parse().ok());
    let mut body = vec![0; length.unwrap_or(0)];
    let _ = reader.read_exact(&mut body);
    let request = Received { path, headers, body: String::from_utf8_lossy(&body).into_owned() };
    log.lock().expect("lock").push(request.clone());
    let (status, body) = match handler(&request) {
        Reply::Status(status, body) => (status, body),
        Reply::Late(delay, body) => {
            thread::sleep(delay);
            (200, body)
        }
    };
    let mut stream = stream;
    let response = format!(
        "HTTP/1.1 {status} Stub\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let _ = stream.write_all(response.as_bytes());
}

/// Settings for the local server with a chosen model.
pub(crate) fn local_settings() -> AiSettings {
    AiSettings { model: Some("sample-model".to_owned()), ..AiSettings::default() }
}

pub(crate) fn tiny_image() -> PageImage {
    PageImage { bytes: vec![1, 2, 3], mime_type: "image/jpeg".to_owned() }
}

pub(crate) fn invoices() -> Category {
    Category::new(CategoryId::new("invoices").expect("id"), "Invoices", None, None).expect("category")
}

pub(crate) fn energy() -> Category {
    Category::new(CategoryId::new("energy").expect("id"), "Energy", Some(invoices().id), None).expect("category")
}

/// An outline with `Home` and `Home/Power`, without the inbox.
pub(crate) fn outline() -> FolderOutline {
    FolderOutline {
        folders: vec![
            OutlineFolder {
                path: path("Home"),
                document_count: 3,
                top_category: Some(energy().id),
                sample_titles: vec!["Sample bill".to_owned()],
                hidden_subfolders: 0,
            },
            OutlineFolder {
                path: path("Home/Power"),
                document_count: 2,
                top_category: Some(energy().id),
                sample_titles: Vec::new(),
                hidden_subfolders: 4,
            },
        ],
        excluded: vec![path("Inbox")],
    }
}

pub(crate) fn path(raw: &str) -> RelativePath {
    RelativePath::of(raw).expect("path")
}

pub(crate) fn tag(raw: &str) -> TagName {
    TagName::of(raw).expect("tag")
}

/// A classification request with synthetic names only.
pub(crate) fn request() -> ClassificationRequest {
    ClassificationRequest {
        document_id: DocumentId::new("doc-1").expect("id"),
        current_file_name: "scan-001.pdf".to_owned(),
        content: DocumentContent::Text("Sample text".to_owned()),
        categories: vec![invoices(), energy()],
        tags: IndexSet::from([tag("Tax")]),
        folders: vec![path("letters")],
        file_name_examples: vec![
            "2024-01-02 Sample - One.pdf".to_owned(),
            "2024-02-03 Sample - Two.pdf".to_owned(),
            "2024-03-04 Sample - Three.pdf".to_owned(),
        ],
        today: NaiveDate::from_ymd_opt(2026, 1, 15).expect("date"),
        outline: None,
        field_definitions: Vec::<FieldDefinition>::new(),
        default_currency: Some(CurrencyCode::new("EUR").expect("currency")),
        fixed_values: None,
    }
}
