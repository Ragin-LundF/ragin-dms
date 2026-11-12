//! Maps between the catalog and its file. The file is sorted (documents by path, categories by name, tags
//! alphabetically) so that saving an unchanged catalog writes the same bytes, and a diff of two versions shows
//! only what changed. Strings sort by UTF-16 code units, as in Kotlin.

use std::collections::{HashMap, HashSet};

use dms_domain::catalog::{Catalog, CatalogRecords, CatalogRevision, LibraryId};
use dms_domain::document::{ClassificationRecord, ContentHash, Document, DocumentId, DocumentMetadata};
use dms_domain::library::RelativePath;
use dms_domain::link::DocumentLink;
use dms_domain::retention::Retention;
use dms_domain::taxonomy::{Category, CategoryId, TagName};
use dms_domain::{iso_date, text};
use indexmap::{IndexMap, IndexSet};

use crate::catalog::CatalogMigrations;
use crate::catalog::dto::{CatalogDto, CategoryDto, ClassificationDto, DocumentDto, LinkDto};
use crate::catalog::mapper::records::{case_mapper, contract_mapper, tax_mapper};
use crate::catalog::mapper::{field_mapper, query_mapper, settings_mapper, unknown_fields};
use crate::enum_names::{self, CLASSIFICATION_SOURCES};
use crate::instant;
use crate::yaml::YamlError;

/// The file content for `catalog`; fields only known to newer versions are taken over from `previous`.
pub fn to_dto(catalog: &Catalog, previous: Option<&CatalogDto>) -> CatalogDto {
    let mut categories: Vec<&Category> = catalog.categories().values().collect();
    categories.sort_by(|first, second| {
        text::compare(&text::lowercase(&first.name), &text::lowercase(&second.name))
            .then_with(|| text::compare(first.id.value(), second.id.value()))
    });
    let mut categories: Vec<CategoryDto> = categories.into_iter().map(category_to_dto).collect();
    let previous_documents: HashMap<&str, &DocumentDto> = previous
        .map(|previous| previous.documents.iter().map(|document| (document.id.as_str(), document)).collect())
        .unwrap_or_default();
    let mut documents: Vec<&Document> = catalog.documents().values().collect();
    documents.sort_by(|first, second| text::compare(first.path.value(), second.path.value()));
    let documents = documents
        .into_iter()
        .map(|document| {
            let previous_document = previous_documents.get(document.id.value()).copied();
            let mut mapped = document_to_dto(document, previous_document);
            unknown_fields::carry(&mut mapped, previous_document);
            mapped
        })
        .collect();
    let mut links: Vec<&DocumentLink> = catalog.links().iter().collect();
    links.sort_by(|first, second| {
        text::compare(first.first().value(), second.first().value())
            .then_with(|| text::compare(first.second().value(), second.second().value()))
    });
    let mut links: Vec<LinkDto> = links
        .into_iter()
        .map(|link| LinkDto {
            first: link.first().value().to_owned(),
            second: link.second().value().to_owned(),
            label: link.label().map(str::to_owned),
            unknown: IndexMap::new(),
        })
        .collect();
    if let Some(previous) = previous {
        unknown_fields::carry_all(&mut categories, &previous.categories, |category| category.id.clone());
        unknown_fields::carry_all(&mut links, &previous.links, |link| (link.first.clone(), link.second.clone()));
    }
    let mut dto = CatalogDto {
        format_version: CatalogMigrations::CURRENT_VERSION,
        library_id: catalog.library_id().value().to_owned(),
        revision: i64::try_from(catalog.revision().value()).unwrap_or(i64::MAX),
        categories,
        tags: query_mapper::sorted_tags(catalog.tags()),
        documents,
        links,
        field_definitions: Vec::new(),
        saved_searches: Vec::new(),
        rules: Vec::new(),
        correspondents: Vec::new(),
        layout: None,
        owner: None,
        bills_tracked_since: None,
        dismissed_contract_proposals: Vec::new(),
        expectations: Vec::new(),
        emergency_folder: None,
        contracts: contract_mapper::to_dto(catalog.contracts(), previous.map_or(&[], |previous| &previous.contracts)),
        cases: case_mapper::to_dto(catalog.cases(), previous.map_or(&[], |previous| &previous.cases)),
        tax_years: tax_mapper::years_to_dto(catalog.tax_years(), previous.map_or(&[], |previous| &previous.tax_years)),
        unknown: IndexMap::new(),
    };
    settings_mapper::fill_settings(&mut dto, catalog.settings(), previous);
    unknown_fields::carry(&mut dto, previous);
    dto
}

/// The catalog in `dto`. An older version that removed a document or contract keeps the 2.1 parts that referred to
/// it as unknown fields, so such references are dropped here, like links, instead of making the file unreadable.
///
/// # Errors
/// Fails when the file breaks a catalog rule.
pub fn to_domain(dto: &CatalogDto) -> Result<Catalog, YamlError> {
    let documents: Vec<Document> = dto.documents.iter().map(document_to_domain).collect::<Result<_, _>>()?;
    let mut tags: IndexSet<TagName> = query_mapper::tags_of(&dto.tags)?;
    tags.extend(documents.iter().flat_map(|document| document.tags.iter().cloned()));
    let ids: HashSet<&DocumentId> = documents.iter().map(|document| &document.id).collect();
    let links = links_of(dto, &ids)?;
    let categories = dto
        .categories
        .iter()
        .map(|category| {
            let category = category_to_domain(category)?;
            Ok((category.id.clone(), category))
        })
        .collect::<Result<IndexMap<_, _>, YamlError>>()?;
    let revision = u64::try_from(dto.revision).map_err(|_| YamlError("Revision must not be negative".to_owned()))?;
    let known_document = |id: &DocumentId| ids.contains(id);
    let records = CatalogRecords {
        contracts: contract_mapper::to_domain(&dto.contracts)?,
        cases: case_mapper::to_domain(&dto.cases)?,
        tax_years: tax_mapper::years_to_domain(&dto.tax_years)?,
    }
    .retaining_documents(&known_document);
    let mut settings = settings_mapper::settings_to_domain(dto)?;
    settings.emergency_folder = settings
        .emergency_folder
        .map(|folder| folder.retaining(&known_document, &|id| records.contracts.contains_key(id)));
    Ok(Catalog::new(
        LibraryId::new(dto.library_id.as_str())?,
        CatalogRevision::new(revision),
        documents.into_iter().map(|document| (document.id.clone(), document)).collect(),
        categories,
        tags,
        links,
        settings,
    )?
    .with_records(records)?)
}

/// The links between documents the file has. An older version that removed a document keeps the links it did not
/// understand, so links to missing documents are dropped here instead of making the whole file unreadable.
fn links_of(dto: &CatalogDto, documents: &HashSet<&DocumentId>) -> Result<IndexSet<DocumentLink>, YamlError> {
    let mut pairs = HashSet::new();
    let mut links = IndexSet::new();
    for link in &dto.links {
        let (first, second) = (DocumentId::new(link.first.as_str())?, DocumentId::new(link.second.as_str())?);
        if !documents.contains(&first) || !documents.contains(&second) || first == second {
            continue;
        }
        let label = link.label.as_deref().filter(|label| !text::is_blank(label));
        let link = DocumentLink::new(first, second, label)?;
        if pairs.insert((link.first().clone(), link.second().clone())) {
            links.insert(link);
        }
    }
    Ok(links)
}

fn category_to_dto(category: &Category) -> CategoryDto {
    CategoryDto {
        id: category.id.value().to_owned(),
        name: category.name.clone(),
        parent_id: category.parent_id.as_ref().map(|parent| parent.value().to_owned()),
        retention_years: category.retention_years.map(|years| i32::try_from(years).unwrap_or(i32::MAX)),
        unknown: IndexMap::new(),
    }
}

fn category_to_domain(category: &CategoryDto) -> Result<Category, YamlError> {
    let retention_years = category
        .retention_years
        .map(|years| u32::try_from(years).map_err(|_| YamlError(format!("Retention years out of range: {years}"))))
        .transpose()?;
    Ok(Category {
        id: CategoryId::new(category.id.as_str())?,
        name: category.name.clone(),
        parent_id: category.parent_id.as_deref().map(CategoryId::new).transpose()?,
        retention_years,
    })
}

fn document_to_dto(document: &Document, previous: Option<&DocumentDto>) -> DocumentDto {
    DocumentDto {
        id: document.id.value().to_owned(),
        path: document.path.value().to_owned(),
        content_hash: document.content_hash.value().to_owned(),
        size: i64::try_from(document.size).unwrap_or(i64::MAX),
        added_at: instant::format(document.added_at),
        title: document.metadata.title.clone(),
        document_date: document.metadata.document_date.map(iso_date),
        correspondent: document.metadata.correspondent.clone(),
        category_id: document.category_id.as_ref().map(|id| id.value().to_owned()),
        tags: query_mapper::sorted_tags(&document.tags),
        classification: document.classification.as_ref().map(|record| ClassificationDto {
            source: enum_names::name_of(CLASSIFICATION_SOURCES, record.source).to_owned(),
            model: record.model.clone(),
            confirmed_at: instant::format(record.confirmed_at),
        }),
        fields: field_mapper::values_to_dto(&document.fields),
        note: document.note.clone(),
        retention: document.retention.as_ref().map(|retention| {
            let mut dto = crate::catalog::dto::RetentionDto {
                keep_until: retention.keep_until.map(iso_date),
                settled_at: retention.settled_at.map(instant::format),
                unknown: IndexMap::new(),
            };
            unknown_fields::carry(&mut dto, previous.and_then(|previous| previous.retention.as_ref()));
            dto
        }),
        tax: document
            .tax
            .as_ref()
            .map(|tax| tax_mapper::assignment_to_dto(tax, previous.and_then(|previous| previous.tax.as_ref()))),
        unknown: IndexMap::new(),
    }
}

fn document_to_domain(document: &DocumentDto) -> Result<Document, YamlError> {
    let size = u64::try_from(document.size).map_err(|_| YamlError("Size must not be negative".to_owned()))?;
    let retention = document
        .retention
        .as_ref()
        .map(|retention| {
            Ok::<_, YamlError>(Retention {
                keep_until: retention.keep_until.as_deref().map(field_mapper::date_of).transpose()?,
                settled_at: retention.settled_at.as_deref().map(instant::parse).transpose()?,
            })
        })
        .transpose()?
        .filter(|retention| !retention.is_empty());
    Ok(Document {
        id: DocumentId::new(document.id.as_str())?,
        path: RelativePath::of(&document.path)?,
        content_hash: ContentHash::from_hex(&document.content_hash)?,
        size,
        added_at: instant::parse(&document.added_at)?,
        metadata: DocumentMetadata {
            title: document.title.clone(),
            document_date: document.document_date.as_deref().map(field_mapper::date_of).transpose()?,
            correspondent: document.correspondent.clone(),
        },
        category_id: document.category_id.as_deref().map(CategoryId::new).transpose()?,
        tags: query_mapper::tags_of(&document.tags)?,
        classification: document
            .classification
            .as_ref()
            .map(|record| {
                Ok::<_, YamlError>(ClassificationRecord {
                    source: enum_names::parse(CLASSIFICATION_SOURCES, &record.source)?,
                    model: record.model.clone(),
                    confirmed_at: instant::parse(&record.confirmed_at)?,
                })
            })
            .transpose()?,
        fields: field_mapper::values_to_domain(&document.fields)?,
        note: document.note.clone().filter(|note| !text::is_blank(note)),
        retention,
        tax: document.tax.as_ref().map(tax_mapper::assignment_to_domain).transpose()?,
    })
}
