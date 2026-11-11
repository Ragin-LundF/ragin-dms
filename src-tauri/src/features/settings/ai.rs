//! The AI settings card: the typed draft checked against the saved settings (Kotlin `AiDraftModel`), and the
//! words for a connection test.

use dms_application::port::ai::{ClassificationFailure, ModelList};
use dms_application::port::state::{AiProvider, AiSettings};

use super::dto::{AiConnectionDto, AiDraftCheckDto, AiDraftDto, AiProviderDto, AiSettingsDto};
use crate::dto::IpcError;

pub fn ai_settings_dto(settings: &AiSettings) -> AiSettingsDto {
    AiSettingsDto {
        enabled: settings.enabled,
        base_url: settings.base_url.clone(),
        model: settings.model.clone(),
        prefer_vision_for_pdfs: settings.prefer_vision_for_pdfs,
        timeout_seconds: settings.timeout_seconds,
        accept_suggested_file_names: settings.accept_suggested_file_names,
        embedding_model: settings.embedding_model.clone(),
        input_characters: settings.input_characters,
        provider: match settings.provider {
            AiProvider::Local => AiProviderDto::Local,
            AiProvider::Anthropic => AiProviderDto::Anthropic,
        },
        anthropic_model: settings.anthropic_model.clone(),
    }
}

/// The settings a command received.
///
/// # Errors
/// `invalid_input` when the settings break a rule, such as a text size outside the range.
pub fn ai_settings_of(dto: AiSettingsDto) -> Result<AiSettings, IpcError> {
    let settings = AiSettings {
        enabled: dto.enabled,
        base_url: dto.base_url,
        model: dto.model,
        prefer_vision_for_pdfs: dto.prefer_vision_for_pdfs,
        timeout_seconds: dto.timeout_seconds,
        accept_suggested_file_names: dto.accept_suggested_file_names,
        embedding_model: dto.embedding_model,
        input_characters: dto.input_characters,
        provider: match dto.provider {
            AiProviderDto::Local => AiProvider::Local,
            AiProviderDto::Anthropic => AiProvider::Anthropic,
        },
        anthropic_model: dto.anthropic_model,
    };
    settings.validate().map_err(|error| IpcError::invalid(error.to_string()))?;
    if settings.timeout_seconds <= 0 {
        return Err(IpcError::invalid("Enter a timeout above 0 seconds"));
    }
    Ok(settings)
}

/// Checks the typed `draft` against the `saved` settings: what is wrong, and what "Save" would write.
pub fn check_draft(draft: &AiDraftDto, saved: &AiSettingsDto) -> AiDraftCheckDto {
    let address_valid = address_valid(draft);
    let timeout = parse_number(&draft.timeout_seconds);
    let input = parse_number(&draft.input_characters);
    let timeout_valid = timeout.is_some_and(|value| value > 0);
    let input_valid = input.is_some_and(|value| AiSettings::INPUT_CHARACTERS_RANGE.contains(&value));
    let problem = if !address_valid {
        Some("Enter a server address starting with http:// or https://".to_owned())
    } else if !timeout_valid {
        Some("Enter a timeout above 0 seconds".to_owned())
    } else if !input_valid {
        let range = AiSettings::INPUT_CHARACTERS_RANGE;
        Some(format!("Enter a text size from {} to {} characters", range.start(), range.end()))
    } else {
        None
    };
    let settings = match (&problem, timeout, input) {
        (None, Some(timeout), Some(input)) => Some(typed_settings(draft, timeout, input)),
        _ => None,
    };
    let save_disabled_reason = match &settings {
        None => problem.clone(),
        Some(typed) if typed == saved => Some("Nothing changed".to_owned()),
        Some(_) => None,
    };
    AiDraftCheckDto { address_valid, timeout_valid, input_valid, problem, settings, save_disabled_reason }
}

/// The address is only checked for Local AI; Anthropic's address is fixed and its field hidden.
fn address_valid(draft: &AiDraftDto) -> bool {
    let url = dms_domain::text::trim(&draft.base_url);
    draft.provider != AiProviderDto::Local || url.starts_with("http://") || url.starts_with("https://")
}

/// Kotlin's `trim().toIntOrNull()`: an optional sign and digits that fit an `Int`.
fn parse_number(text: &str) -> Option<i32> {
    dms_domain::text::trim(text).parse().ok()
}

fn typed_settings(draft: &AiDraftDto, timeout_seconds: i32, input_characters: i32) -> AiSettingsDto {
    let optional = |text: &str| Some(dms_domain::text::trim(text).to_owned()).filter(|value| !value.is_empty());
    AiSettingsDto {
        enabled: draft.enabled,
        base_url: dms_domain::text::trim(&draft.base_url).trim_end_matches('/').to_owned(),
        model: optional(&draft.model),
        prefer_vision_for_pdfs: draft.prefer_vision_for_pdfs,
        timeout_seconds,
        accept_suggested_file_names: draft.accept_suggested_file_names,
        embedding_model: optional(&draft.embedding_model),
        input_characters,
        provider: draft.provider,
        anthropic_model: optional(&draft.anthropic_model)
            .unwrap_or_else(|| AiSettings::DEFAULT_ANTHROPIC_MODEL.to_owned()),
    }
}

/// What a connection test found, in words.
pub fn present_connection(list: &ModelList) -> AiConnectionDto {
    match list {
        ModelList::Available(models) if models.is_empty() => AiConnectionDto {
            models: Vec::new(),
            message: "The server answers, but has no model loaded. Load one in LM Studio".to_owned(),
            is_error: true,
        },
        ModelList::Available(models) => AiConnectionDto {
            models: models.clone(),
            message: format!("Connected. The server offers {} models", models.len()),
            is_error: false,
        },
        ModelList::Unavailable { reason, detail } => AiConnectionDto {
            models: Vec::new(),
            message: format!(
                "{}{}",
                ai_failure(*reason),
                detail.as_ref().map(|detail| format!(" ({detail})")).unwrap_or_default()
            ),
            is_error: true,
        },
    }
}

/// Kotlin `AiMessages.failure`.
// shortcut: lives here until Bob moves the AI messages to messages.rs.
pub fn ai_failure(reason: ClassificationFailure) -> &'static str {
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
    use super::*;

    fn saved() -> AiSettingsDto {
        ai_settings_dto(&AiSettings::default())
    }

    fn draft_of(settings: &AiSettingsDto) -> AiDraftDto {
        AiDraftDto {
            enabled: settings.enabled,
            base_url: settings.base_url.clone(),
            model: settings.model.clone().unwrap_or_default(),
            timeout_seconds: settings.timeout_seconds.to_string(),
            prefer_vision_for_pdfs: settings.prefer_vision_for_pdfs,
            accept_suggested_file_names: settings.accept_suggested_file_names,
            embedding_model: settings.embedding_model.clone().unwrap_or_default(),
            input_characters: settings.input_characters.to_string(),
            provider: settings.provider,
            anthropic_model: settings.anthropic_model.clone(),
        }
    }

    #[test]
    fn an_untouched_draft_has_nothing_to_save() {
        let check = check_draft(&draft_of(&saved()), &saved());
        assert_eq!(check.problem, None);
        assert_eq!(check.save_disabled_reason.as_deref(), Some("Nothing changed"));
    }

    #[test]
    fn problems_are_named_in_the_order_address_timeout_text_size() {
        let mut draft = AiDraftDto {
            base_url: "localhost:1234".to_owned(),
            timeout_seconds: "0".to_owned(),
            input_characters: "999".to_owned(),
            ..draft_of(&saved())
        };
        let check = check_draft(&draft, &saved());
        assert_eq!(check.problem.as_deref(), Some("Enter a server address starting with http:// or https://"));
        assert_eq!(check.save_disabled_reason, check.problem);
        assert!(check.settings.is_none());
        draft.base_url = "https://example.test/v1".to_owned();
        assert_eq!(check_draft(&draft, &saved()).problem.as_deref(), Some("Enter a timeout above 0 seconds"));
        draft.timeout_seconds = "30".to_owned();
        assert_eq!(
            check_draft(&draft, &saved()).problem.as_deref(),
            Some("Enter a text size from 1000 to 200000 characters")
        );
        draft.input_characters = "200001".to_owned();
        assert!(!check_draft(&draft, &saved()).input_valid);
    }

    #[test]
    fn the_address_is_not_checked_for_anthropic() {
        let draft = AiDraftDto { base_url: String::new(), provider: AiProviderDto::Anthropic, ..draft_of(&saved()) };
        let check = check_draft(&draft, &saved());
        assert!(check.address_valid);
        assert_eq!(check.problem, None);
    }

    #[test]
    fn saving_trims_the_address_and_blanks_become_defaults() {
        let draft = AiDraftDto {
            enabled: true,
            base_url: " http://localhost:1234/v1/ ".to_owned(),
            model: "  ".to_owned(),
            embedding_model: String::new(),
            anthropic_model: " ".to_owned(),
            ..draft_of(&saved())
        };
        let typed = check_draft(&draft, &saved()).settings.expect("valid");
        assert_eq!(typed.base_url, "http://localhost:1234/v1");
        assert_eq!(typed.model, None);
        assert_eq!(typed.embedding_model, None);
        assert_eq!(typed.anthropic_model, "claude-opus-5-5");
        assert!(typed.enabled);
    }

    #[test]
    fn a_test_names_the_models_or_why_the_server_did_not_answer() {
        let found = present_connection(&ModelList::Available(vec!["a".to_owned(), "b".to_owned()]));
        assert_eq!(found.message, "Connected. The server offers 2 models");
        assert!(!found.is_error);
        let empty = present_connection(&ModelList::Available(Vec::new()));
        assert!(empty.is_error);
        let failed = present_connection(&ModelList::Unavailable {
            reason: ClassificationFailure::Timeout,
            detail: Some("10 s".to_owned()),
        });
        assert_eq!(failed.message, "The AI took too long; try again, or raise the timeout in Settings › AI (10 s)");
    }

    #[test]
    fn settings_outside_the_text_range_are_refused() {
        let settings = AiSettingsDto { input_characters: 10, ..saved() };
        assert!(ai_settings_of(settings).is_err());
        assert!(ai_settings_of(saved()).is_ok());
    }
}
