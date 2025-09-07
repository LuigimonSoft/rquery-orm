pub mod db;
pub mod infrastructure;
pub mod mapping;
pub mod query;
pub mod repository;
pub mod services;

pub use db::{connect_mssql, connect_postgres, DatabaseRef, DbKind};
pub use infrastructure::generic_repository::GenericRepository;
pub use mapping::{
    ColumnMeta, Entity, FromRowNamed, KeyAsGuid, KeyAsInt, KeyAsString, KeyMeta, Persistable,
    RelationMeta, TableMeta, Validatable,
};
pub use query::{Expr, JoinType, PlaceholderStyle, Query, SqlParam, ToParam};
pub use repository::{Crud, QueryExecutor, Repository};

pub use rquery_orm_macros::Entity; // derive macro
