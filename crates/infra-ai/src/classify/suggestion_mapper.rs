//! Turns the model's answer into a suggestion the domain accepts. A value that does not fit is dropped rather than
//! failing the whole suggestion: an unknown date format, an invalid file name, a blank tag.

use dms_application::port::ai::ClassificationRequest;
use dms_domain::bill::Iban;
use dms_domain::classification::{
    ClassificationSuggestion, RequestedAction, SuggestedCategory, SuggestedFolder, SuggestedTag,
};
use dms_domain::field::{FieldKey, FieldPresets, FieldType, FieldValue, FieldValues};
use dms_domain::library::{FileNamePolicy, RelativePath};
use dms_domain::money::Money;
use dms_domain::tax::{TaxArea, TaxAssignment};
use dms_domain::taxonomy::{Category, TagName};
use dms_domain::{parse_iso_date, text};
use indexmap::{IndexMap, IndexSet};

use super::prompt_texts::path_of;
use super::suggestion_dto::{FieldAnswerDto, RequestedActionDto, SuggestionDto};
use crate::prompt::is_tax_tag;

const MAX_TAGS: usize = 8;
const PAGE_BREAK: &str = "\u{c}";
const MAX_EXTENSION: usize = 5;
const MAX_REQUESTED_ACTIONS: usize = 5;

pub(crate) fn to_suggestion(
    answer: &SuggestionDto,
    request: &ClassificationRequest,
    model: &str,
) -> ClassificationSuggestion {
    let mut suggestion = ClassificationSuggestion::empty(request.document_id.clone(), model);
    suggestion.title = cleaned(answer.title.as_deref());
    suggestion.category = cleaned(answer.category.as_deref()).and_then(|name| category_of(&name, &request.categories));
    suggestion.tags = tags_of(&answer.tags, &request.tags);
    suggestion.document_date = cleaned(answer.document_date.as_deref()).and_then(|date| parse_iso_date(&date));
    suggestion.correspondent = cleaned(answer.correspondent.as_deref());
    suggestion.file_name =
        cleaned(answer.file_name.as_deref()).and_then(|name| file_name_of(&name, &request.current_file_name));
    suggestion.transcription = transcription_of(answer);
    suggestion.confidence = answer.confidence.map(|confidence| confidence.clamp(0.0, 1.0));
    suggestion.target_folder = target_folder_of(answer, request);
    suggestion.fields = fields_of(&answer.fields, request);
    suggestion.fields.extend(payment_fields(answer));
    suggestion.target_reason = cleaned(answer.target_reason.as_deref()).map(|reason| first_line(&reason).to_owned());
    if suggestion.tags.iter().any(|tag| is_tax_tag(tag.name.value())) {
        suggestion.tax = tax_of(answer, request);
    }
    suggestion.requested_actions = requested_actions_of(&answer.requested_actions);
    suggestion
}

const TAX_AREAS: [(&str, TaxArea); 9] = [
    ("income", TaxArea::Income),
    ("pensionInsurance", TaxArea::PensionInsurance),
    ("householdServices", TaxArea::HouseholdServices),
    ("donations", TaxArea::Donations),
    ("healthCosts", TaxArea::HealthCosts),
    ("workExpenses", TaxArea::WorkExpenses),
    ("capitalIncome", TaxArea::CapitalIncome),
    ("rental", TaxArea::Rental),
    ("other", TaxArea::Other),
];

/// The tax year and area, both needed; the deductible part only for household services and craftsmen, where it is
/// the labour share, and only as an amount of money.
fn tax_of(answer: &SuggestionDto, request: &ClassificationRequest) -> Option<TaxAssignment> {
    let year: i32 = cleaned(answer.tax_year.as_deref())?.parse().ok()?;
    let area_name = cleaned(answer.tax_area.as_deref())?;
    let area = TAX_AREAS.iter().find(|(name, _)| name.eq_ignore_ascii_case(&area_name)).map(|(_, area)| *area)?;
    let deductible = cleaned(answer.deductible.as_deref())
        .filter(|_| area == TaxArea::HouseholdServices)
        .and_then(|raw| FieldValues::parse(FieldType::Money, &raw, request.default_currency.as_ref()))
        .as_ref()
        .and_then(Money::of_field);
    let tax = TaxAssignment { year, area, deductible };
    tax.validate().is_ok().then_some(tax)
}

/// The requested actions with a text, at most [`MAX_REQUESTED_ACTIONS`]; a due date that is no date is dropped,
/// the action kept.
fn requested_actions_of(answers: &[RequestedActionDto]) -> Vec<RequestedAction> {
    answers
        .iter()
        .filter_map(|answer| {
            Some(RequestedAction {
                text: first_line(&cleaned(answer.text.as_deref())?).to_owned(),
                due_date: cleaned(answer.due_date.as_deref()).and_then(|date| parse_iso_date(&date)),
                references: answer.references.iter().filter_map(|reference| cleaned(Some(reference))).collect(),
            })
        })
        .take(MAX_REQUESTED_ACTIONS)
        .collect()
}

/// The text read on the page images: the pages separated by a form feed, so a text layer can put each on its own
/// page, or the whole transcription when the model gave no pages.
fn transcription_of(answer: &SuggestionDto) -> Option<String> {
    let pages: Vec<&str> = answer.pages.iter().map(|page| text::trim(page)).collect();
    if pages.iter().any(|page| !page.is_empty()) {
        return Some(pages.join(PAGE_BREAK));
    }
    cleaned(answer.transcription.as_deref())
}

/// The values of defined fields that fit their type; anything else is dropped, never guessed.
fn fields_of(answers: &[FieldAnswerDto], request: &ClassificationRequest) -> IndexMap<FieldKey, FieldValue> {
    answers
        .iter()
        .filter_map(|answer| {
            let definition = request
                .field_definitions
                .iter()
                .find(|definition| definition.key.value() == text::trim(&answer.key))?;
            let raw = cleaned(answer.value.as_deref())?;
            let value = FieldValues::parse(definition.field_type, &raw, request.default_currency.as_ref())?;
            Some((definition.key.clone(), value))
        })
        .collect()
}

/// The IBAN (only one that passes its checksum) and the payment reference printed on a bill, as the standard bill
/// fields.
fn payment_fields(answer: &SuggestionDto) -> Vec<(FieldKey, FieldValue)> {
    let iban = cleaned(answer.iban.as_deref())
        .and_then(|iban| Iban::parse(&iban).ok())
        .map(|iban| (FieldPresets::iban().key, FieldValue::Text(iban.value().to_owned())));
    let reference = cleaned(answer.payment_reference.as_deref()).map(|reference| {
        (FieldPresets::payment_reference().key, FieldValue::Text(text::trim(first_line(&reference)).to_owned()))
    });
    iban.into_iter().chain(reference).collect()
}

/// A folder of the outline, or a new folder directly below one that the model flagged as new; any other path, and
/// every path when no outline was sent, is dropped.
fn target_folder_of(answer: &SuggestionDto, request: &ClassificationRequest) -> Option<SuggestedFolder> {
    let outline = request.outline.as_ref()?;
    let raw = cleaned(answer.target_folder.as_deref())?.replace('\\', "/");
    let segments: Vec<&str> = raw.split('/').map(text::trim).filter(|segment| !segment.is_empty()).collect();
    if segments.is_empty() || segments.iter().any(|segment| FileNamePolicy::check(segment).is_some()) {
        return None;
    }
    let path = RelativePath::from_segments(&segments).ok()?;
    if outline.contains(&path) {
        Some(SuggestedFolder { path, is_new: false })
    } else if answer.target_folder_is_new == Some(true) && outline.can_create(&path) {
        Some(SuggestedFolder { path, is_new: true })
    } else {
        None
    }
}

/// An existing category by name or path, ignoring case; otherwise a new one, below its parent if named.
pub(crate) fn category_of(name: &str, categories: &[Category]) -> Option<SuggestedCategory> {
    let key = text::lowercase(name);
    let path_key = |category: &Category| text::lowercase(&path_of(category, categories));
    if let Some(existing) =
        categories.iter().find(|category| text::lowercase(&category.name) == key || path_key(category) == key)
    {
        return Some(SuggestedCategory::Existing(existing.id.clone()));
    }
    let (parent_name, own_name) = name.rsplit_once('/').unwrap_or(("", name));
    let parent_key = text::lowercase(text::trim(parent_name));
    let parent = categories.iter().find(|category| !parent_key.is_empty() && path_key(category) == parent_key);
    let own_name = text::trim(own_name);
    (!own_name.is_empty()).then(|| SuggestedCategory::New {
        name: own_name.to_owned(),
        parent_id: parent.map(|parent| parent.id.clone()),
    })
}

/// Parsed, distinct, at most [`MAX_TAGS`]; an existing tag keeps the catalog's spelling.
fn tags_of(names: &[String], existing: &IndexSet<TagName>) -> Vec<SuggestedTag> {
    let mut tags: Vec<TagName> = Vec::new();
    for tag in names.iter().filter_map(|raw| TagName::parse(raw)) {
        if !tags.contains(&tag) {
            tags.push(tag);
        }
    }
    tags.into_iter()
        .take(MAX_TAGS)
        .map(|tag| match existing.get(&tag) {
            Some(known) => SuggestedTag { name: known.clone(), is_new: false },
            None => SuggestedTag { name: tag, is_new: true },
        })
        .collect()
}

/// Keeps the file's extension, and drops a name the file name policy refuses or that is the current name.
fn file_name_of(name: &str, current: &str) -> Option<String> {
    let extension = current.rsplit_once('.').map_or("", |(_, extension)| extension);
    let with_extension = if extension.is_empty() || ends_with_ignoring_case(name, &format!(".{extension}")) {
        name.to_owned()
    } else {
        match name.rsplit_once('.') {
            Some((base, other)) if is_extension(other) => format!("{base}.{extension}"),
            _ => format!("{name}.{extension}"),
        }
    };
    let with_extension = text::trim(&with_extension);
    (FileNamePolicy::check(with_extension).is_none() && with_extension != current).then(|| with_extension.to_owned())
}

fn is_extension(text: &str) -> bool {
    (1..=MAX_EXTENSION).contains(&text.len()) && text.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

fn ends_with_ignoring_case(text: &str, suffix: &str) -> bool {
    text::lowercase(text).ends_with(&text::lowercase(suffix))
}

/// The first line, as Kotlin's `lines().first()`.
fn first_line(text: &str) -> &str {
    text.split(['\r', '\n']).next().unwrap_or_default()
}

/// Trimmed; empty and the word "null" count as missing.
fn cleaned(value: Option<&str>) -> Option<String> {
    let value = text::trim(value?);
    (!value.is_empty() && !value.eq_ignore_ascii_case("null")).then(|| value.to_owned())
}
