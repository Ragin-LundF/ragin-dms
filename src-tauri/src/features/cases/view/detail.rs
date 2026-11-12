use chrono::NaiveDate;
use dms_domain::cases::{Case, CaseStatus, EntryDirection, FollowUp, FollowUpSource};
use dms_domain::catalog::Catalog;
use dms_domain::document::DocumentId;
use dms_domain::iso_date;
use indexmap::IndexMap;

use super::document_name;
use super::list::{pill_of, waiting_for};
use crate::features::cases::dto::{
    CaseDetailDto, CaseLetterDto, CaseRequestDto, CaseTimelineItemDto, CaseTimelineKindDto,
};

pub(super) fn present_detail(catalog: &Catalog, case: &Case, today: NaiveDate) -> CaseDetailDto {
    CaseDetailDto {
        id: case.id.to_string(),
        title: case.title.clone(),
        summary: summary_of(case, today),
        pill: pill_of(case, today),
        requests: requests_of(catalog, case),
        timeline: timeline_of(catalog, case, today),
        reminder: reminder_of(catalog, case),
    }
}

/// "Tax office · 3 documents · Waiting for an answer for 24 days".
fn summary_of(case: &Case, today: NaiveDate) -> String {
    let documents = match case.entries.len() {
        1 => "1 document".to_owned(),
        count => format!("{count} documents"),
    };
    let waiting =
        case.waiting_since.filter(|_| case.status == CaseStatus::Waiting).map(|since| waiting_for(since, today));
    case.correspondent.iter().cloned().chain([documents]).chain(waiting).collect::<Vec<_>>().join(" · ")
}

/// The open follow-ups the AI proposed, by the letter that asked for them.
fn requests_of(catalog: &Catalog, case: &Case) -> Vec<CaseRequestDto> {
    let mut by_letter: IndexMap<&DocumentId, Vec<String>> = IndexMap::new();
    for follow_up in case.open_follow_ups() {
        if let FollowUpSource::Ai { document_id } = &follow_up.source {
            by_letter.entry(document_id).or_default().push(follow_up.text.clone());
        }
    }
    by_letter
        .into_iter()
        .map(|(document_id, actions)| CaseRequestDto {
            document_id: document_id.to_string(),
            document_name: document_name(catalog, document_id),
            actions,
        })
        .collect()
}

/// Documents, follow-ups and notes by date; on the same day documents first, then follow-ups, then notes.
fn timeline_of(catalog: &Catalog, case: &Case, today: NaiveDate) -> Vec<CaseTimelineItemDto> {
    let entries = case.entries.iter().map(|entry| {
        let (kind, word) = match entry.direction {
            EntryDirection::Received => (CaseTimelineKindDto::Received, "received"),
            EntryDirection::Sent => (CaseTimelineKindDto::Sent, "sent"),
        };
        CaseTimelineItemDto {
            document_id: Some(entry.document_id.to_string()),
            ..item(kind, entry.date, word, document_name(catalog, &entry.document_id))
        }
    });
    let follow_ups = case.follow_ups.iter().map(|follow_up| follow_up_item(catalog, follow_up, today));
    let notes = case.notes.iter().map(|note| item(CaseTimelineKindDto::Note, note.date, "note", note.text.clone()));
    let mut timeline: Vec<CaseTimelineItemDto> = entries.chain(follow_ups).chain(notes).collect();
    timeline.sort_by(|left, right| left.date.cmp(&right.date));
    timeline
}

fn follow_up_item(catalog: &Catalog, follow_up: &FollowUp, today: NaiveDate) -> CaseTimelineItemDto {
    let detail = match (&follow_up.source, follow_up.done) {
        (_, true) => Some("Done".to_owned()),
        (FollowUpSource::Ai { document_id }, false) => {
            Some(format!("Proposed by the AI from {}", document_name(catalog, document_id)))
        }
        (FollowUpSource::Manual, false) => None,
    };
    CaseTimelineItemDto {
        detail,
        follow_up_id: Some(follow_up.id.to_string()),
        done: follow_up.done,
        due: !follow_up.done && follow_up.date <= today,
        ..item(CaseTimelineKindDto::FollowUp, follow_up.date, "follow-up", follow_up.text.clone())
    }
}

fn item(kind: CaseTimelineKindDto, date: NaiveDate, word: &str, title: String) -> CaseTimelineItemDto {
    CaseTimelineItemDto {
        kind,
        date: iso_date(date),
        marker: format!("{} · {word}", iso_date(date)),
        title,
        detail: None,
        document_id: None,
        follow_up_id: None,
        done: false,
        due: false,
    }
}

/// The case's documents still in the library, to the correspondent.
fn reminder_of(catalog: &Catalog, case: &Case) -> CaseLetterDto {
    CaseLetterDto {
        case_id: case.id.to_string(),
        document_ids: case
            .referenced_documents()
            .filter(|id| catalog.documents().contains_key(*id))
            .map(ToString::to_string)
            .collect(),
        recipient: case.correspondent.clone(),
    }
}
