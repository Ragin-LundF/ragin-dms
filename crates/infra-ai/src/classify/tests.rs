#![allow(clippy::expect_used)]

use chrono::NaiveDate;
use dms_application::port::ai::{
    AiServer, ClassificationFailure, ClassificationRequest, ClassificationResult, DocumentClassifier, DocumentContent,
    ModelList,
};
use dms_application::port::state::{AiProvider, AiSettings};
use dms_domain::classification::{ClassificationSuggestion, RequestedAction, SuggestedCategory, SuggestedFolder};
use dms_domain::field::{CurrencyCode, Decimal, FieldKey, FieldPresets, FieldValue};
use dms_domain::rule::RuleResult;
use indexmap::{IndexMap, IndexSet};
use serde_json::json;

use super::suggestion_dto::{FieldAnswerDto, SuggestionDto};
use super::{naming_convention, prompt_texts, suggestion_mapper};
use crate::test_support::{
    StubServer, adapters_with, chat_answer, energy, invoices, local_settings, ok, outline, path, request, tag,
    tiny_image, unreachable_base_url,
};

fn suggested(result: ClassificationResult) -> ClassificationSuggestion {
    match result {
        ClassificationResult::Suggested(suggestion) => suggestion,
        ClassificationResult::Failed { reason, detail } => panic!("failed: {reason:?} {detail:?}"),
    }
}

fn failure(result: ClassificationResult) -> (ClassificationFailure, Option<String>) {
    match result {
        ClassificationResult::Failed { reason, detail } => (reason, detail),
        ClassificationResult::Suggested(suggestion) => panic!("suggested: {suggestion:?}"),
    }
}

fn anthropic() -> AiSettings {
    AiSettings {
        provider: AiProvider::Anthropic,
        anthropic_model: "claude-opus-5-5".to_owned(),
        ..AiSettings::default()
    }
}

// --- the classifier over HTTP ---

#[test]
fn the_request_carries_the_model_the_prompt_and_the_answer_schema() {
    let server = StubServer::start(|_| ok(chat_answer(r#"{"title":"Sample bill","tags":["Tax"]}"#)));
    let ai = server.adapters(local_settings(), None);

    let suggestion = suggested(ai.client.suggest(&request()));

    assert_eq!(suggestion.title.as_deref(), Some("Sample bill"));
    assert_eq!(suggestion.model, "sample-model");
    let body = &server.chat_bodies()[0];
    assert_eq!(body["model"], "sample-model");
    assert_eq!(body["response_format"]["type"], "json_schema");
    assert_eq!(body["response_format"]["json_schema"]["name"], "document_classification");
    assert_eq!(body["temperature"], 0.1);
    assert!(body["messages"][0]["content"].as_str().expect("system").contains("Invoices / Energy"));
}

#[test]
fn a_letter_that_asks_for_something_answers_its_requested_actions_and_the_schema_asks_for_them() {
    let answer = r#"{"title":"Request for receipts","requestedActions":[
        {"text":" Send the plumber invoices ","dueDate":"2026-11-30","references":["R-2026-114"," "]},
        {"text":"Reply","dueDate":"by the end of the month","references":null},
        {"text":" ","dueDate":null,"references":[]}]}"#;
    let server = StubServer::start(move |_| ok(chat_answer(answer)));
    let ai = server.adapters(local_settings(), None);

    let suggestion = suggested(ai.client.suggest(&request()));

    assert_eq!(
        suggestion.requested_actions,
        vec![
            RequestedAction {
                text: "Send the plumber invoices".to_owned(),
                due_date: NaiveDate::from_ymd_opt(2026, 11, 30),
                references: vec!["R-2026-114".to_owned()],
            },
            RequestedAction { text: "Reply".to_owned(), due_date: None, references: Vec::new() },
        ]
    );
    let schema = &server.chat_bodies()[0]["response_format"]["json_schema"]["schema"];
    assert!(schema["required"].as_array().expect("required").contains(&json!("requestedActions")));
    let prompt = server.chat_bodies()[0]["messages"][0]["content"].as_str().expect("system").to_owned();
    assert!(prompt.contains("requestedActions: only if the document is a letter that asks the person"));
}

#[test]
fn page_images_are_sent_as_data_urls_after_the_text() {
    let server = StubServer::start(|_| ok(chat_answer("{}")));
    let ai = server.adapters(local_settings(), None);
    let request = ClassificationRequest { content: DocumentContent::PageImages(vec![tiny_image()]), ..request() };

    ai.client.suggest(&request);

    let parts = server.chat_bodies()[0]["messages"][1]["content"].as_array().expect("parts").clone();
    let types: Vec<_> = parts.iter().map(|part| part["type"].as_str().expect("type")).collect();
    assert_eq!(types, ["text", "image_url"]);
    assert_eq!(parts[1]["image_url"]["url"], "data:image/jpeg;base64,AQID");
}

#[test]
fn an_answer_wrapped_in_a_code_fence_is_still_read_and_prose_alone_is_invalid() {
    let fenced = StubServer::start(|_| ok(chat_answer("```json\n{\"title\":\"Sample\"}\n```")));
    let prose = StubServer::start(|_| ok(chat_answer("I cannot tell.")));

    suggested(fenced.adapters(local_settings(), None).client.suggest(&request()));
    let (reason, detail) = failure(prose.adapters(local_settings(), None).client.suggest(&request()));

    assert_eq!(reason, ClassificationFailure::InvalidResponse);
    assert_eq!(detail.as_deref(), Some("I cannot tell."));
}

#[test]
fn an_answer_with_a_wrong_type_is_invalid_not_partly_used() {
    let server = StubServer::start(|_| ok(chat_answer(r#"{"title":{"text":"Sample"},"tags":["Tax"]}"#)));

    let (reason, _) = failure(server.adapters(local_settings(), None).client.suggest(&request()));

    assert_eq!(reason, ClassificationFailure::InvalidResponse);
}

#[test]
fn a_server_that_cannot_be_reached_is_reported_as_unavailable() {
    let unreachable = AiSettings { base_url: unreachable_base_url(), ..local_settings() };
    let ai = adapters_with(unreachable.clone(), None, &unreachable_base_url());

    let (reason, _) = failure(ai.client.suggest(&request()));
    let models = ai.client.models(&unreachable);

    assert_eq!(reason, ClassificationFailure::ProviderUnavailable);
    assert!(matches!(models, ModelList::Unavailable { reason: ClassificationFailure::ProviderUnavailable, .. }));
}

#[test]
fn without_a_chosen_model_the_first_loaded_one_is_used() {
    let server = StubServer::start(|request| {
        if request.path.ends_with("/models") {
            ok(r#"{"data":[{"id":"first-model"},{"id":"second-model"}]}"#)
        } else {
            ok(chat_answer("{}"))
        }
    });

    let suggestion =
        suggested(server.adapters(AiSettings { model: None, ..local_settings() }, None).client.suggest(&request()));

    assert_eq!(suggestion.model, "first-model");
}

#[test]
fn a_reasoning_models_json_in_the_reasoning_content_is_used_when_the_answer_is_empty() {
    let body = json!({"choices": [{"message": {"role": "assistant", "content": "", "reasoning_content": "{\"title\":\"Sample bill\"}"}}]});
    let server = StubServer::start(move |_| ok(body.to_string()));

    let suggestion = suggested(server.adapters(local_settings(), None).client.suggest(&request()));

    assert_eq!(suggestion.title.as_deref(), Some("Sample bill"));
}

#[test]
fn anthropic_gets_a_messages_api_request_with_the_key_and_its_text_block_is_the_answer() {
    let text = json!({"content": [{"type": "thinking", "thinking": ""}, {"type": "text", "text": "{\"title\":\"Sample bill\"}"}]});
    let server = StubServer::start(move |_| ok(text.to_string()));

    let suggestion = suggested(server.adapters(anthropic(), Some("sample-key")).client.suggest(&request()));

    assert_eq!(suggestion.title.as_deref(), Some("Sample bill"));
    let received = server.received();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].path, "/anthropic/v1/messages");
    assert_eq!(received[0].header("x-api-key"), Some("sample-key"));
    assert_eq!(received[0].header("anthropic-version"), Some("2023-06-01"));
    assert!(server.chat_bodies()[0].get("output_config").is_some());
}

#[test]
fn anthropic_without_an_api_key_fails_before_any_request() {
    let server = StubServer::start(|_| ok("{}"));

    let blank = failure(server.adapters(anthropic(), Some("  ")).client.suggest(&request()));
    let missing = failure(server.adapters(anthropic(), None).client.suggest(&request()));

    assert_eq!(missing, (ClassificationFailure::ProviderUnavailable, Some("ANTHROPIC_API_KEY is not set".to_owned())));
    assert_eq!(blank, missing);
    assert!(server.received().is_empty());
}

#[test]
fn an_anthropic_refusal_is_a_failure() {
    let server = StubServer::start(|_| ok(r#"{"content":[],"stop_reason":"refusal"}"#));

    let (reason, detail) = failure(server.adapters(anthropic(), Some("sample-key")).client.suggest(&request()));

    assert_eq!(reason, ClassificationFailure::InvalidResponse);
    assert_eq!(detail.as_deref(), Some("The AI declined"));
}

#[test]
fn anthropic_http_400_with_images_is_no_vision_problem() {
    let server = StubServer::start(|_| crate::test_support::Reply::Status(400, "image too large".to_owned()));
    let request = ClassificationRequest { content: DocumentContent::PageImages(vec![tiny_image()]), ..request() };

    let (reason, detail) = failure(server.adapters(anthropic(), Some("sample-key")).client.suggest(&request));

    assert_eq!(reason, ClassificationFailure::ProviderUnavailable);
    assert_eq!(detail.as_deref(), Some("HTTP 400: image too large"));
}

#[test]
fn the_anthropic_key_is_never_sent_to_the_local_server() {
    let server = StubServer::start(|request| {
        if request.path.ends_with("/models") { ok(r#"{"data":[{"id":"first-model"}]}"#) } else { ok(chat_answer("{}")) }
    });

    server.adapters(AiSettings { model: None, ..local_settings() }, Some("sample-key")).client.suggest(&request());

    assert_eq!(server.paths(), ["/v1/models", "/v1/chat/completions"]);
    assert!(server.received().iter().all(|request| request.header("x-api-key").is_none()));
}

#[test]
fn anthropics_model_list_is_read_with_the_key_and_without_a_key_it_is_unavailable() {
    let server = StubServer::start(|_| ok(r#"{"data":[{"id":"claude-opus-5-5"}]}"#));
    let settings = AiSettings { provider: AiProvider::Anthropic, ..AiSettings::default() };

    let models = server.adapters(settings.clone(), Some("sample-key")).client.models(&settings);
    let missing = server.adapters(settings.clone(), None).client.models(&settings);

    assert_eq!(models, ModelList::Available(vec!["claude-opus-5-5".to_owned()]));
    let received = server.received();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].path, "/anthropic/v1/models");
    assert_eq!(received[0].header("x-api-key"), Some("sample-key"));
    assert_eq!(
        missing,
        ModelList::Unavailable {
            reason: ClassificationFailure::ProviderUnavailable,
            detail: Some("ANTHROPIC_API_KEY is not set".to_owned()),
        }
    );
}

#[test]
fn a_model_list_that_is_no_model_list_is_unavailable() {
    let server = StubServer::start(|_| ok("<html>Not found</html>"));
    let settings = AiSettings { base_url: server.base_url(), ..AiSettings::default() };

    let models = server.adapters(settings.clone(), None).client.models(&settings);

    assert!(matches!(models, ModelList::Unavailable { reason: ClassificationFailure::ProviderUnavailable, .. }));
}

// --- naming convention ---

#[test]
fn a_date_prefix_and_a_separator_are_recognized() {
    let examples =
        ["2024-01-02 Sample - One.pdf", "2024-02-03 Sample - Two.pdf", "2024-03-04 X - Y.pdf"].map(String::from);

    assert_eq!(
        naming_convention::describe(&examples),
        "Most file names start with the document date as YYYY-MM-DD and separate their parts with \" - \"."
    );
}

#[test]
fn year_and_month_underscores_and_no_pattern_are_told_apart() {
    let months = ["2024-01_a.pdf", "2024-02_b.pdf", "2024-03_c.pdf"].map(String::from);
    let mixed = ["alpha.pdf", "beta.pdf", "gamma.pdf"].map(String::from);
    let compact = ["20240102.pdf", "20240203.pdf", "20240304.pdf"].map(String::from);

    assert_eq!(
        naming_convention::describe(&months),
        "Most file names start with year and month as YYYY-MM and separate their parts with underscores."
    );
    assert_eq!(naming_convention::describe(&mixed), "No common pattern; follow the examples.");
    assert_eq!(naming_convention::describe(&compact), "Most file names start with the document date as YYYYMMDD.");
}

#[test]
fn too_few_examples_give_no_convention() {
    assert_eq!(
        naming_convention::describe(&["a.pdf".to_owned()]),
        "The library has too few files to show a convention. Use a short, descriptive name."
    );
}

// --- prompt texts ---

#[test]
fn the_system_prompt_is_filled_from_its_template_with_the_taxonomy() {
    let system = prompt_texts::system(&request());

    assert!(!system.contains("{{"));
    assert!(system.contains("Existing categories: Invoices; Invoices / Energy"));
    assert!(system.contains("\".pdf\""));
    assert!(system.contains("\"Tax\""));
    assert!(system.contains("Today is 2026-01-15."));
    assert!(system.starts_with("You file documents"), "the template is trimmed");
}

#[test]
fn the_user_prompt_carries_the_file_name_and_the_text_as_the_reader_cut_it() {
    let text = "x".repeat(500);

    let user = prompt_texts::user(&request(), Some(&text));

    assert!(!user.contains("{{"));
    assert!(user.starts_with("Current file name: scan-001.pdf"));
    assert!(user.ends_with(&text));
}

#[test]
fn document_text_that_looks_like_a_placeholder_is_not_filled() {
    let user = prompt_texts::user(&request(), Some("Dear {{fileName}}"));

    assert!(user.ends_with("Dear {{fileName}}"));
}

#[test]
fn the_outline_lists_folders_with_counts_categories_and_samples_or_asks_for_no_folder() {
    let with_outline = prompt_texts::user(&ClassificationRequest { outline: Some(outline()), ..request() }, Some("x"));
    let without = prompt_texts::user(&request(), Some("x"));

    assert!(with_outline.contains("Home | 3 documents | mostly Invoices / Energy | e.g. Sample bill"));
    assert!(with_outline.contains("Home/Power | 2 documents | mostly Invoices / Energy | 4 more subfolders"));
    assert!(without.contains("answer targetFolder with null"));
}

#[test]
fn the_system_prompt_lists_the_custom_fields_but_never_a_retention() {
    let system = prompt_texts::system(&ClassificationRequest { field_definitions: FieldPresets::all(), ..request() });

    assert!(system.contains("amount (Amount, MONEY in EUR unless stated otherwise)"));
    assert!(system.contains("cancel-by (Cancel by, DATE)"));
    assert!(!system.to_lowercase().contains("retention"));
}

#[test]
fn values_the_rules_decided_are_named_as_fixed_and_nothing_is_said_without_rules() {
    let fixed = RuleResult {
        correspondent: Some("Sample Energy".to_owned()),
        category_id: Some(energy().id),
        tags: IndexSet::from([tag("Tax")]),
        ..RuleResult::default()
    };

    let section = prompt_texts::rules_section(&ClassificationRequest { fixed_values: Some(fixed), ..request() });

    assert_eq!(
        section,
        "Already decided by the person's rules, keep these values: category: Invoices / Energy; \
         correspondent: Sample Energy; tags: Tax"
    );
    assert_eq!(prompt_texts::rules_section(&request()), "");
}

// --- mapping the answer ---

#[allow(clippy::needless_pass_by_value)] // keeps the many call sites short
fn map(answer: SuggestionDto) -> ClassificationSuggestion {
    suggestion_mapper::to_suggestion(&answer, &request(), "sample-model")
}

#[test]
fn an_existing_category_is_found_by_name_or_path_ignoring_case() {
    let by_path = map(SuggestionDto { category: Some("invoices / ENERGY".to_owned()), ..SuggestionDto::default() });
    let by_name = map(SuggestionDto { category: Some("invoices".to_owned()), ..SuggestionDto::default() });

    assert_eq!(by_path.category, Some(SuggestedCategory::Existing(energy().id)));
    assert_eq!(by_name.category, Some(SuggestedCategory::Existing(invoices().id)));
}

#[test]
fn an_unknown_category_is_new_below_its_parent_if_the_path_names_one() {
    let water = map(SuggestionDto { category: Some("Invoices / Water".to_owned()), ..SuggestionDto::default() });
    let contracts = map(SuggestionDto { category: Some("Contracts".to_owned()), ..SuggestionDto::default() });

    assert_eq!(
        water.category,
        Some(SuggestedCategory::New { name: "Water".to_owned(), parent_id: Some(invoices().id) })
    );
    assert_eq!(contracts.category, Some(SuggestedCategory::New { name: "Contracts".to_owned(), parent_id: None }));
}

#[test]
fn tags_keep_the_catalog_spelling_and_say_whether_they_are_new() {
    let tags = map(SuggestionDto {
        tags: ["tax", " ", "Home", "home"].map(String::from).to_vec(),
        ..SuggestionDto::default()
    })
    .tags;

    let names: Vec<_> = tags.iter().map(|suggested| suggested.name.value()).collect();
    assert_eq!(names, ["Tax", "Home"]);
    assert_eq!(tags.iter().map(|suggested| suggested.is_new).collect::<Vec<_>>(), [false, true]);
}

#[test]
fn at_most_eight_tags_are_suggested() {
    let many: Vec<String> = (1..=10).map(|number| format!("tag {number}")).collect();

    assert_eq!(map(SuggestionDto { tags: many, ..SuggestionDto::default() }).tags.len(), 8);
}

#[test]
fn values_that_do_not_fit_are_dropped_not_the_whole_suggestion() {
    let suggestion = map(SuggestionDto {
        title: Some("  Sample bill ".to_owned()),
        document_date: Some("15.01.2026".to_owned()),
        correspondent: Some("null".to_owned()),
        file_name: Some("a:b".to_owned()),
        confidence: Some(1.7),
        ..SuggestionDto::default()
    });

    assert_eq!(suggestion.title.as_deref(), Some("Sample bill"));
    assert_eq!(suggestion.document_date, None);
    assert_eq!(suggestion.correspondent, None);
    assert_eq!(suggestion.file_name, None);
    assert_eq!(suggestion.confidence, Some(1.0));
    assert!(suggestion.validate().is_ok());
}

#[test]
fn the_file_name_keeps_the_extension_of_the_file() {
    let file_name =
        |name: &str| map(SuggestionDto { file_name: Some(name.to_owned()), ..SuggestionDto::default() }).file_name;
    let dated = map(SuggestionDto { document_date: Some("2026-01-15".to_owned()), ..SuggestionDto::default() });

    assert_eq!(file_name("2026-01-15 Sample").as_deref(), Some("2026-01-15 Sample.pdf"));
    assert_eq!(file_name("2026-01-15 Sample.docx").as_deref(), Some("2026-01-15 Sample.pdf"));
    assert_eq!(file_name("Sample.PDF").as_deref(), Some("Sample.PDF"));
    assert_eq!(file_name("scan-001"), None, "the current name is no suggestion");
    assert_eq!(dated.document_date, NaiveDate::from_ymd_opt(2026, 1, 15));
}

#[test]
fn a_target_folder_must_be_in_the_outline_or_a_new_folder_flagged_as_new_below_one() {
    let with_outline = ClassificationRequest { outline: Some(outline()), ..request() };
    let target = |folder: &str, is_new: Option<bool>| {
        let answer = SuggestionDto {
            target_folder: Some(folder.to_owned()),
            target_folder_is_new: is_new,
            target_reason: Some("Same sender\nmore".to_owned()),
            ..SuggestionDto::default()
        };
        suggestion_mapper::to_suggestion(&answer, &with_outline, "m")
    };

    assert_eq!(
        target("/Home/Power/", Some(false)).target_folder,
        Some(SuggestedFolder { path: path("Home/Power"), is_new: false })
    );
    assert_eq!(
        target("Home\\Water", Some(true)).target_folder,
        Some(SuggestedFolder { path: path("Home/Water"), is_new: true })
    );
    assert_eq!(target("Home/Water", Some(false)).target_folder, None);
    assert_eq!(target("Away/Water", Some(true)).target_folder, None);
    assert_eq!(target("Inbox/Sorted", Some(true)).target_folder, None);
    assert_eq!(target("Home/Wa:ter", Some(true)).target_folder, None);
    assert_eq!(target("Home", None).target_reason.as_deref(), Some("Same sender"));
}

#[test]
fn field_values_must_be_defined_and_fit_their_type_a_missing_currency_takes_the_default() {
    let with_fields = ClassificationRequest { field_definitions: FieldPresets::all(), ..request() };
    let field =
        |key: &str, value: Option<&str>| FieldAnswerDto { key: key.to_owned(), value: value.map(str::to_owned) };
    let answer = SuggestionDto {
        fields: vec![
            field("amount", Some("1.234,50")),
            field("due", Some("next week")),
            field(" cancel-by ", Some("2026-09-30")),
            field("unknown", Some("x")),
            field("warranty-until", None),
        ],
        ..SuggestionDto::default()
    };

    let fields = suggestion_mapper::to_suggestion(&answer, &with_fields, "m").fields;

    let expected = IndexMap::from([
        (
            FieldPresets::amount().key,
            FieldValue::Money {
                amount: Decimal::parse("1234.50").expect("decimal"),
                currency: CurrencyCode::new("EUR").expect("currency"),
            },
        ),
        (FieldPresets::cancel_by().key, FieldValue::Date(NaiveDate::from_ymd_opt(2026, 9, 30).expect("date"))),
    ]);
    assert_eq!(fields, expected);
}

#[test]
fn a_printed_iban_and_payment_reference_become_bill_fields_and_an_invalid_iban_is_dropped() {
    let answer = |iban: &str| SuggestionDto {
        iban: Some(iban.to_owned()),
        payment_reference: Some(" KD 4711 \nsecond line".to_owned()),
        ..SuggestionDto::default()
    };

    let valid = suggestion_mapper::to_suggestion(&answer("de89 3704 0044 0532 0130 00"), &request(), "m").fields;
    let invalid = suggestion_mapper::to_suggestion(&answer("DE89 3704 0044 0532 0130 01"), &request(), "m").fields;

    assert_eq!(
        valid,
        IndexMap::from([
            (FieldPresets::iban().key, FieldValue::Text("DE89370400440532013000".to_owned())),
            (FieldPresets::payment_reference().key, FieldValue::Text("KD 4711".to_owned())),
        ])
    );
    assert_eq!(invalid.keys().map(FieldKey::value).collect::<Vec<_>>(), ["payment-reference"]);
}

#[test]
fn the_answer_schema_asks_for_iban_and_payment_reference() {
    let schema = super::suggestion_schema::suggestion_schema();
    let required: Vec<&str> =
        schema["required"].as_array().expect("required").iter().filter_map(|name| name.as_str()).collect();

    assert!(required.contains(&"iban") && required.contains(&"paymentReference"));
    assert_eq!(schema["properties"]["iban"]["type"], json!(["string", "null"]));
}

#[test]
fn page_texts_become_one_transcription_with_a_form_feed_between_pages() {
    let paged = map(SuggestionDto {
        transcription: Some("all".to_owned()),
        pages: vec!["first".to_owned(), " second ".to_owned()],
        ..SuggestionDto::default()
    });
    let whole = map(SuggestionDto {
        transcription: Some("all".to_owned()),
        pages: vec![" ".to_owned()],
        ..SuggestionDto::default()
    });

    assert_eq!(paged.transcription.as_deref(), Some("first\u{c}second"));
    assert_eq!(whole.transcription.as_deref(), Some("all"));
}

#[test]
fn without_an_outline_no_target_folder_is_suggested() {
    let suggestion = map(SuggestionDto {
        target_folder: Some("Home".to_owned()),
        target_folder_is_new: Some(false),
        ..SuggestionDto::default()
    });

    assert_eq!(suggestion.target_folder, None);
}

#[test]
fn a_lenient_answer_with_nulls_and_numbers_is_read() {
    let answer =
        SuggestionDto::parse(r#"Here: {"title": 2024, "tags": null, "fields": null, "confidence": null} done"#)
            .expect("parsed");

    assert_eq!(answer.title.as_deref(), Some("2024"));
    assert!(answer.tags.is_empty() && answer.fields.is_empty());
    assert!(SuggestionDto::parse("no json").is_none());
    assert!(SuggestionDto::parse(r#"{"tags": "Tax"}"#).is_none());
}

#[test]
fn a_tax_document_gets_its_year_area_and_labour_share_and_others_get_no_tax() {
    let tax_answer = |area: &str| SuggestionDto {
        tags: vec!["Tax".to_owned()],
        tax_year: Some("2025".to_owned()),
        tax_area: Some(area.to_owned()),
        deductible: Some("684.50 EUR".to_owned()),
        ..SuggestionDto::default()
    };

    let craftsman = map(tax_answer("householdServices")).tax.expect("a tax assignment");
    let donation = map(tax_answer("Donations")).tax.expect("a tax assignment");
    let untagged = map(SuggestionDto { tags: vec!["Home".to_owned()], ..tax_answer("donations") });
    let unknown_area = map(tax_answer("holidays"));
    let wrong_year = map(SuggestionDto { tax_year: Some("25".to_owned()), ..tax_answer("donations") });

    assert_eq!((craftsman.year, craftsman.area), (2025, dms_domain::tax::TaxArea::HouseholdServices));
    let labour = craftsman.deductible.expect("the labour share");
    assert_eq!((labour.amount, labour.currency.value()), (Decimal::parse("684.50").expect("amount"), "EUR"));
    assert_eq!(donation.deductible, None, "a deductible part only for household services");
    assert_eq!(untagged.tax, None, "only with the tax tag");
    assert_eq!(unknown_area.tax, None);
    assert_eq!(wrong_year.tax, None);
}

#[test]
fn the_answer_schema_asks_for_the_tax_values() {
    let schema = super::suggestion_schema::suggestion_schema();
    let required: Vec<&str> =
        schema["required"].as_array().expect("required").iter().filter_map(|name| name.as_str()).collect();
    assert!(["taxYear", "taxArea", "deductible"].iter().all(|name| required.contains(name)));
    assert_eq!(schema["properties"]["taxYear"]["type"], json!(["integer", "null"]));
}
