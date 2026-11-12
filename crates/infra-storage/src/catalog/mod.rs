//! `dms-catalog.yaml`: reading, writing, conflict copies and format migrations.

mod catalog_files;
mod catalog_migrations;
pub mod dto;
mod file_catalog_repository;
pub mod mapper;

pub use catalog_files::CatalogFiles;
pub use catalog_migrations::CatalogMigrations;
pub use file_catalog_repository::FileCatalogRepository;
pub(crate) use file_catalog_repository::read_catalog;
