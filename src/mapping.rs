use crate::query::{PlaceholderStyle, SqlParam};

pub struct ColumnMeta {
    pub name: &'static str,
    pub required: bool,
    pub allow_null: bool,
    pub max_length: Option<usize>,
    pub min_length: Option<usize>,
    pub allow_empty: bool,
    pub regex: Option<&'static str>,
    pub error_max_length: Option<&'static str>,
    pub error_min_length: Option<&'static str>,
    pub error_required: Option<&'static str>,
    pub error_allow_null: Option<&'static str>,
    pub error_allow_empty: Option<&'static str>,
    pub error_regex: Option<&'static str>,
    pub ignore: bool,
    pub ignore_in_update: bool,
    pub ignore_in_insert: bool,
    pub ignore_in_delete: bool,
}

pub struct KeyMeta {
    pub column: &'static str,
    pub is_identity: bool,
    pub ignore_in_update: bool,
    pub ignore_in_insert: bool,
}

pub struct RelationMeta {
    pub name: &'static str,
    pub foreign_key: &'static str,
    pub table: &'static str,
    pub table_number: Option<u32>,
    pub ignore_in_update: bool,
    pub ignore_in_insert: bool,
}

pub struct TableMeta {
    pub name: &'static str,
    pub schema: Option<&'static str>,
    pub columns: &'static [ColumnMeta],
    pub keys: &'static [KeyMeta],
    pub relations: &'static [RelationMeta],
}

pub trait Entity {
    fn table() -> &'static TableMeta;
}

pub trait FromRowNamed: Sized {
    fn from_row_ms(row: &tiberius::Row) -> anyhow::Result<Self>;
    fn from_row_pg(row: &tokio_postgres::Row) -> anyhow::Result<Self>;
}

pub trait Validatable {
    fn validate(&self) -> Result<(), Vec<String>>;
}

pub trait Persistable {
    fn build_insert(&self, style: PlaceholderStyle) -> (String, Vec<SqlParam>, bool);
    fn build_update(&self, style: PlaceholderStyle) -> (String, Vec<SqlParam>);
    fn build_delete(&self, style: PlaceholderStyle) -> (String, Vec<SqlParam>);
    fn build_delete_by_key(key: SqlParam, style: PlaceholderStyle) -> (String, Vec<SqlParam>);
}

pub trait KeyAsInt {
    fn key(&self) -> i32;
}

pub trait KeyAsGuid {
    fn key(&self) -> uuid::Uuid;
}

pub trait KeyAsString {
    fn key(&self) -> String;
}
