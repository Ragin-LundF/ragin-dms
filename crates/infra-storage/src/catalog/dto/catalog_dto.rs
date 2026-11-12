use indexmap::IndexMap;

use crate::catalog::dto::case::CaseDto;
use crate::catalog::dto::contract::ContractDto;
use crate::catalog::dto::organize::{EmergencyFolderDto, ExpectationDto, OwnerDto};
use crate::catalog::dto::rule::RuleDto;
use crate::catalog::dto::settings::{CorrespondentDto, FieldDefinitionDto, LayoutDto, SavedSearchDto};
use crate::catalog::dto::tax::TaxYearDto;
use crate::catalog::dto::{CategoryDto, DocumentDto, LinkDto};
use crate::yaml::{Fields, MapWriter, Node, YamlError};

use super::{extensible, owned, owned_texts, read_list, texts_node};

/// The catalog file `dms-catalog.yaml`. Everything after `documents` was added in 1.1 as optional fields, so
/// older versions keep it as unknown fields and older files still load; the same holds for the parts after `links`,
/// added in 2.1. `NON_EMPTY`.
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogDto {
    pub format_version: i32,
    pub library_id: String,
    pub revision: i64,
    pub categories: Vec<CategoryDto>,
    pub tags: Vec<String>,
    pub documents: Vec<DocumentDto>,
    pub links: Vec<LinkDto>,
    pub field_definitions: Vec<FieldDefinitionDto>,
    pub saved_searches: Vec<SavedSearchDto>,
    pub rules: Vec<RuleDto>,
    pub correspondents: Vec<CorrespondentDto>,
    pub layout: Option<LayoutDto>,
    pub owner: Option<OwnerDto>,
    pub bills_tracked_since: Option<String>,
    pub dismissed_contract_proposals: Vec<String>,
    pub expectations: Vec<ExpectationDto>,
    pub emergency_folder: Option<EmergencyFolderDto>,
    pub contracts: Vec<ContractDto>,
    pub cases: Vec<CaseDto>,
    pub tax_years: Vec<TaxYearDto>,
    pub unknown: IndexMap<String, Node>,
}

extensible!(CatalogDto);

const KNOWN: [&str; 20] = [
    "formatVersion",
    "libraryId",
    "revision",
    "layout",
    "fieldDefinitions",
    "savedSearches",
    "correspondents",
    "rules",
    "categories",
    "tags",
    "documents",
    "links",
    "owner",
    "billsTrackedSince",
    "dismissedContractProposals",
    "expectations",
    "emergencyFolder",
    "contracts",
    "cases",
    "taxYears",
];

impl CatalogDto {
    pub fn read(node: &Node) -> Result<Self, YamlError> {
        let fields = Fields::of(node)?;
        let format_version = fields.opt_i32("formatVersion")?.ok_or_else(|| missing("formatVersion"))?;
        Ok(Self {
            format_version,
            library_id: fields.text("libraryId")?.to_owned(),
            revision: fields.i64("revision")?,
            categories: read_list(fields, "categories", CategoryDto::read)?,
            tags: owned_texts(fields, "tags")?,
            documents: read_list(fields, "documents", DocumentDto::read)?,
            links: read_list(fields, "links", LinkDto::read)?,
            field_definitions: read_list(fields, "fieldDefinitions", FieldDefinitionDto::read)?,
            saved_searches: read_list(fields, "savedSearches", SavedSearchDto::read)?,
            rules: read_list(fields, "rules", RuleDto::read)?,
            correspondents: read_list(fields, "correspondents", CorrespondentDto::read)?,
            layout: fields.get("layout").map(LayoutDto::read).transpose()?,
            owner: fields.get("owner").map(OwnerDto::read).transpose()?,
            bills_tracked_since: owned(fields.opt_text("billsTrackedSince")?),
            dismissed_contract_proposals: owned_texts(fields, "dismissedContractProposals")?,
            expectations: read_list(fields, "expectations", ExpectationDto::read)?,
            emergency_folder: fields.get("emergencyFolder").map(EmergencyFolderDto::read).transpose()?,
            contracts: read_list(fields, "contracts", ContractDto::read)?,
            cases: read_list(fields, "cases", CaseDto::read)?,
            tax_years: read_list(fields, "taxYears", TaxYearDto::read)?,
            unknown: fields.unknown(&KNOWN),
        })
    }

    pub fn write(&self) -> Node {
        MapWriter::new()
            .int("formatVersion", self.format_version)
            .non_empty_text("libraryId", Some(&self.library_id))
            .int("revision", self.revision)
            .opt_node("layout", self.layout.as_ref().map(LayoutDto::write))
            .non_empty_list("fieldDefinitions", self.field_definitions.iter().map(FieldDefinitionDto::write).collect())
            .non_empty_list("savedSearches", self.saved_searches.iter().map(SavedSearchDto::write).collect())
            .non_empty_list("correspondents", self.correspondents.iter().map(CorrespondentDto::write).collect())
            .non_empty_list("rules", self.rules.iter().map(RuleDto::write).collect())
            .non_empty_list("categories", self.categories.iter().map(CategoryDto::write).collect())
            .non_empty_list("tags", texts_node(&self.tags))
            .non_empty_list("documents", self.documents.iter().map(DocumentDto::write).collect())
            .non_empty_list("links", self.links.iter().map(LinkDto::write).collect())
            .opt_node("owner", self.owner.as_ref().map(OwnerDto::write))
            .non_empty_text("billsTrackedSince", self.bills_tracked_since.as_deref())
            .non_empty_list("dismissedContractProposals", texts_node(&self.dismissed_contract_proposals))
            .non_empty_list("expectations", self.expectations.iter().map(ExpectationDto::write).collect())
            .opt_node("emergencyFolder", self.emergency_folder.as_ref().map(EmergencyFolderDto::write))
            .non_empty_list("contracts", self.contracts.iter().map(ContractDto::write).collect())
            .non_empty_list("cases", self.cases.iter().map(CaseDto::write).collect())
            .non_empty_list("taxYears", self.tax_years.iter().map(TaxYearDto::write).collect())
            .unknown(&self.unknown)
            .build()
    }
}

fn missing(key: &str) -> YamlError {
    YamlError(format!("missing property '{key}'"))
}
