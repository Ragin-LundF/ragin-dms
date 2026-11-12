//! The library-wide settings. Field definitions, saved searches and rules keep the order the user gave them;
//! correspondents are sorted by name.

use std::collections::HashMap;

use dms_domain::field::CurrencyCode;
use dms_domain::library::RelativePath;
use dms_domain::rule::Correspondent;
use dms_domain::settings::{CatalogSettings, InboxSettings, LibraryLayout};
use dms_domain::text;
use indexmap::IndexMap;

use crate::catalog::dto::CatalogDto;
use crate::catalog::dto::settings::{CorrespondentDto, InboxDto, LayoutDto};
use crate::catalog::mapper::records::organize_mapper;
use crate::catalog::mapper::{field_mapper, query_mapper, rule_mapper, unknown_fields};
use crate::yaml::YamlError;

/// Fills the settings parts of `dto`; fields only newer versions know are taken over from `previous`, per field
/// key, id or name.
pub fn fill_settings(dto: &mut CatalogDto, settings: &CatalogSettings, previous: Option<&CatalogDto>) {
    let mut searches: Vec<_> = settings.saved_searches.iter().map(query_mapper::search_to_dto).collect();
    let previous_searches: HashMap<&str, _> = previous
        .map(|previous| previous.saved_searches.iter().map(|search| (search.id.as_str(), search)).collect())
        .unwrap_or_default();
    for search in &mut searches {
        let previous_query = previous_searches.get(search.id.as_str()).map(|previous| &previous.query);
        unknown_fields::carry(&mut search.query, previous_query);
    }
    let mut rules: Vec<_> = settings.rules.iter().map(rule_mapper::rule_to_dto).collect();
    let previous_rules: HashMap<&str, _> = previous
        .map(|previous| previous.rules.iter().map(|rule| (rule.id.as_str(), rule)).collect())
        .unwrap_or_default();
    for rule in &mut rules {
        let previous_actions = previous_rules.get(rule.id.as_str()).map(|previous| &previous.actions);
        unknown_fields::carry(&mut rule.actions, previous_actions);
    }
    let mut correspondents: Vec<&Correspondent> = settings.correspondents.iter().collect();
    correspondents
        .sort_by(|first, second| text::compare(&text::lowercase(&first.name), &text::lowercase(&second.name)));
    let mut correspondents: Vec<CorrespondentDto> = correspondents
        .into_iter()
        .map(|correspondent| CorrespondentDto {
            name: correspondent.name.clone(),
            aliases: correspondent.aliases.clone(),
            unknown: IndexMap::new(),
        })
        .collect();
    let mut definitions: Vec<_> = settings.field_definitions.iter().map(field_mapper::definition_to_dto).collect();
    if let Some(previous) = previous {
        unknown_fields::carry_all(&mut definitions, &previous.field_definitions, |definition| definition.key.clone());
        unknown_fields::carry_all(&mut searches, &previous.saved_searches, |search| search.id.clone());
        unknown_fields::carry_all(&mut rules, &previous.rules, |rule| rule.id.clone());
        unknown_fields::carry_all(&mut correspondents, &previous.correspondents, |correspondent| {
            correspondent.name.clone()
        });
    }
    dto.field_definitions = definitions;
    dto.saved_searches = searches;
    dto.rules = rules;
    dto.correspondents = correspondents;
    dto.layout = layout_to_dto(&settings.layout, previous.and_then(|previous| previous.layout.as_ref()));
    dto.owner = settings
        .owner
        .as_ref()
        .map(|owner| organize_mapper::owner_to_dto(owner, previous.and_then(|previous| previous.owner.as_ref())));
    dto.bills_tracked_since = settings.bills_tracked_since.map(dms_domain::iso_date);
    dto.dismissed_contract_proposals.clone_from(&settings.dismissed_contract_proposals);
    dto.expectations = organize_mapper::expectations_to_dto(
        &settings.expectations,
        previous.map_or(&[], |previous| &previous.expectations),
    );
    dto.emergency_folder = settings.emergency_folder.as_ref().map(|folder| {
        organize_mapper::emergency_to_dto(folder, previous.and_then(|previous| previous.emergency_folder.as_ref()))
    });
}

pub fn settings_to_domain(dto: &CatalogDto) -> Result<CatalogSettings, YamlError> {
    Ok(CatalogSettings {
        field_definitions: dto
            .field_definitions
            .iter()
            .map(field_mapper::definition_to_domain)
            .collect::<Result<_, _>>()?,
        saved_searches: dto.saved_searches.iter().map(query_mapper::search_to_domain).collect::<Result<_, _>>()?,
        rules: dto.rules.iter().map(rule_mapper::rule_to_domain).collect::<Result<_, _>>()?,
        correspondents: dto
            .correspondents
            .iter()
            .map(|correspondent| Correspondent {
                name: correspondent.name.clone(),
                aliases: correspondent.aliases.clone(),
            })
            .collect(),
        layout: dto.layout.as_ref().map(layout_to_domain).transpose()?.unwrap_or_default(),
        owner: dto.owner.as_ref().map(organize_mapper::owner_to_domain),
        bills_tracked_since: dto.bills_tracked_since.as_deref().map(field_mapper::date_of).transpose()?,
        dismissed_contract_proposals: dto.dismissed_contract_proposals.clone(),
        expectations: organize_mapper::expectations_to_domain(&dto.expectations)?,
        emergency_folder: dto.emergency_folder.as_ref().map(organize_mapper::emergency_to_domain).transpose()?,
    })
}

/// `None` for the default layout, so a library that uses none of it writes nothing, unless the previous file
/// holds fields of a newer version there.
fn layout_to_dto(layout: &LibraryLayout, previous: Option<&LayoutDto>) -> Option<LayoutDto> {
    let previous_inbox = previous.and_then(|previous| previous.inbox.as_ref());
    if *layout == LibraryLayout::default()
        && !unknown_fields::has_any(previous)
        && !unknown_fields::has_any(previous_inbox)
    {
        return None;
    }
    let inbox = &layout.inbox;
    let mut inbox_dto =
        (*inbox != InboxSettings::default() || unknown_fields::has_any(previous_inbox)).then(|| InboxDto {
            enabled: inbox.enabled,
            path: inbox.path.value().to_owned(),
            ai_analysis: inbox.ai_analysis,
            ai_target_folder: inbox.ai_target_folder,
            auto_analyze: inbox.auto_analyze,
            propose_splits: inbox.propose_splits,
            convert_emails: inbox.convert_emails,
            keep_email_originals: inbox.keep_email_originals,
            unknown: IndexMap::new(),
        });
    if let Some(inbox_dto) = &mut inbox_dto {
        unknown_fields::carry(inbox_dto, previous_inbox);
    }
    let mut dto = LayoutDto {
        inbox: inbox_dto,
        path_template: layout.path_template.clone(),
        default_currency: layout.default_currency.as_ref().map(|currency| currency.value().to_owned()),
        embed_metadata: layout.embed_metadata.then_some(true),
        unknown: IndexMap::new(),
    };
    unknown_fields::carry(&mut dto, previous);
    Some(dto)
}

fn layout_to_domain(layout: &LayoutDto) -> Result<LibraryLayout, YamlError> {
    let inbox = layout
        .inbox
        .as_ref()
        .map(|inbox| {
            Ok::<_, YamlError>(InboxSettings {
                enabled: inbox.enabled,
                path: RelativePath::of(&inbox.path)?,
                ai_analysis: inbox.ai_analysis,
                ai_target_folder: inbox.ai_target_folder,
                auto_analyze: inbox.auto_analyze,
                propose_splits: inbox.propose_splits,
                convert_emails: inbox.convert_emails,
                keep_email_originals: inbox.keep_email_originals,
            })
        })
        .transpose()?;
    Ok(LibraryLayout {
        inbox: inbox.unwrap_or_default(),
        path_template: layout.path_template.clone(),
        default_currency: layout.default_currency.as_deref().map(CurrencyCode::new).transpose()?,
        embed_metadata: layout.embed_metadata == Some(true),
    })
}
