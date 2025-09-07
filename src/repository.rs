use async_trait::async_trait;

use crate::mapping::{Entity, FromRowNamed, Persistable, Validatable};
use crate::query::{Query, SqlParam};

#[async_trait]
pub trait QueryExecutor<T>
where
    T: Entity + FromRowNamed + Validatable + Persistable + Send + Sync,
{
    fn Select(&self) -> Query<T>;
    async fn GetByKeyAsync(&self, key: SqlParam) -> anyhow::Result<Option<T>>;
}

#[async_trait]
pub trait Crud<T>
where
    T: Entity + FromRowNamed + Validatable + Persistable + Send + Sync,
{
    async fn InsertAsync(&self, entity: &T) -> anyhow::Result<()>;
    async fn UpdateAsync(&self, entity: &T) -> anyhow::Result<()>;
    async fn DeleteByEntityAsync(&self, entity: &T) -> anyhow::Result<()>;
    async fn DeleteByKeyAsync(&self, key: SqlParam) -> anyhow::Result<()>;
}

pub trait Repository<T>: QueryExecutor<T> + Crud<T>
where
    T: Entity + FromRowNamed + Validatable + Persistable + Send + Sync,
{
}
