use async_trait::async_trait;

use crate::mapping::{Entity, FromRowNamed, Persistable, Validatable};
use crate::query::{Query, SqlParam};

#[allow(non_snake_case)]
#[async_trait]
pub trait QueryExecutor<T>
where
    T: Entity + FromRowNamed + Validatable + Persistable + Send + Sync,
{
    fn Select(&self) -> Query<T>;
    async fn get_by_key_async(&self, key: SqlParam) -> anyhow::Result<Option<T>>;
}

#[async_trait]
pub trait Crud<T>
where
    T: Entity + FromRowNamed + Validatable + Persistable + Send + Sync,
{
    async fn insert_async(&self, entity: &T) -> anyhow::Result<()>;
    async fn update_async(&self, entity: &T) -> anyhow::Result<()>;
    async fn delete_by_entity_async(&self, entity: &T) -> anyhow::Result<()>;
    async fn delete_by_key_async(&self, key: SqlParam) -> anyhow::Result<()>;
}

pub trait Repository<T>: QueryExecutor<T> + Crud<T>
where
    T: Entity + FromRowNamed + Validatable + Persistable + Send + Sync,
{
}
