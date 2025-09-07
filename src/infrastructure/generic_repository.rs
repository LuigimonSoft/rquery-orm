use std::marker::PhantomData;
use std::sync::Arc;

use async_trait::async_trait;

use crate::db::{DatabaseRef, DbKind};
use crate::mapping::{Entity, FromRowNamed, Persistable, Validatable};
use crate::query::{Expr, PlaceholderStyle, Query, SqlParam};
use crate::repository::{Crud, QueryExecutor, Repository};
use anyhow::{anyhow, Result};

pub struct GenericRepository<T>
where
    T: Entity + FromRowNamed + Validatable + Persistable + Send + Sync,
{
    db: Arc<DatabaseRef>,
    _t: PhantomData<T>,
}

impl<T> GenericRepository<T>
where
    T: Entity + FromRowNamed + Validatable + Persistable + Send + Sync,
{
    pub fn new(db: DatabaseRef) -> Self {
        Self {
            db: Arc::new(db),
            _t: PhantomData,
        }
    }
}

impl<T> Clone for GenericRepository<T>
where
    T: Entity + FromRowNamed + Validatable + Persistable + Send + Sync,
{
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            _t: PhantomData,
        }
    }
}

#[async_trait]
impl<T> QueryExecutor<T> for GenericRepository<T>
where
    T: Entity + FromRowNamed + Validatable + Persistable + Send + Sync,
{
    fn Select(&self) -> Query<T> {
        let style = match self.db.as_ref().kind() {
            DbKind::Mssql => PlaceholderStyle::AtP,
            DbKind::Postgres => PlaceholderStyle::Dollar,
        };
        Query::new(T::table().name, style).with_db(self.db.clone())
    }

    async fn GetByKeyAsync(&self, key: SqlParam) -> Result<Option<T>> {
        let table = T::table();
        let pk = table
            .keys
            .first()
            .ok_or_else(|| anyhow!("no primary key metadata"))?;
        let expr = Expr::Col(format!("{}.{}", table.name, pk.column)).eq(Expr::Param(key));
        self.Select().Where(expr).ToSingleAsync().await
    }
}

#[async_trait]
impl<T> Crud<T> for GenericRepository<T>
where
    T: Entity + FromRowNamed + Validatable + Persistable + Send + Sync,
{
    async fn InsertAsync(&self, entity: &T) -> Result<()> {
        entity.validate().map_err(|e| anyhow!(e.join(", ")))?;
        let style = match self.db.as_ref().kind() {
            DbKind::Mssql => PlaceholderStyle::AtP,
            DbKind::Postgres => PlaceholderStyle::Dollar,
        };
        let (sql, params, _has_identity) = entity.build_insert(style);
        execute(&self.db, &sql, &params).await.map(|_| ())
    }

    async fn UpdateAsync(&self, entity: &T) -> Result<()> {
        entity.validate().map_err(|e| anyhow!(e.join(", ")))?;
        let style = match self.db.as_ref().kind() {
            DbKind::Mssql => PlaceholderStyle::AtP,
            DbKind::Postgres => PlaceholderStyle::Dollar,
        };
        let (sql, params) = entity.build_update(style);
        execute(&self.db, &sql, &params).await.map(|_| ())
    }

    async fn DeleteByEntityAsync(&self, entity: &T) -> Result<()> {
        let style = match self.db.as_ref().kind() {
            DbKind::Mssql => PlaceholderStyle::AtP,
            DbKind::Postgres => PlaceholderStyle::Dollar,
        };
        let (sql, params) = entity.build_delete(style);
        execute(&self.db, &sql, &params).await.map(|_| ())
    }

    async fn DeleteByKeyAsync(&self, key: SqlParam) -> Result<()> {
        let style = match self.db.as_ref().kind() {
            DbKind::Mssql => PlaceholderStyle::AtP,
            DbKind::Postgres => PlaceholderStyle::Dollar,
        };
        let (sql, params) = T::build_delete_by_key(key, style);
        execute(&self.db, &sql, &params).await.map(|_| ())
    }
}

impl<T> Repository<T> for GenericRepository<T> where
    T: Entity + FromRowNamed + Validatable + Persistable + Send + Sync
{
}

async fn execute(db: &Arc<DatabaseRef>, sql: &str, params: &[SqlParam]) -> Result<u64> {
    match db.as_ref() {
        DatabaseRef::Mssql(conn) => {
            let mut guard = conn.lock().await;
            let mut boxed: Vec<Box<dyn tiberius::ToSql + Send + Sync>> = Vec::new();
            for p in params {
                let b: Box<dyn tiberius::ToSql + Send + Sync> = match p {
                    SqlParam::I32(v) => Box::new(*v),
                    SqlParam::I64(v) => Box::new(*v),
                    SqlParam::Bool(v) => Box::new(*v),
                    SqlParam::Text(v) => Box::new(v.clone()),
                    SqlParam::Uuid(v) => Box::new(*v),
                    SqlParam::Decimal(v) => Box::new(v.to_string()),
                    SqlParam::DateTime(v) => Box::new(*v),
                    SqlParam::Bytes(v) => Box::new(v.clone()),
                    SqlParam::Null => Box::new(Option::<i32>::None),
                };
                boxed.push(b);
            }
            let refs: Vec<&dyn tiberius::ToSql> =
                boxed.iter().map(|b| &**b as &dyn tiberius::ToSql).collect();
            let res = guard.execute(sql, &refs[..]).await?;
            Ok(res.total())
        }
        DatabaseRef::Postgres(pg) => {
            let mut boxed: Vec<Box<dyn tokio_postgres::types::ToSql + Send + Sync>> = Vec::new();
            for p in params {
                let b: Box<dyn tokio_postgres::types::ToSql + Send + Sync> = match p {
                    SqlParam::I32(v) => Box::new(*v),
                    SqlParam::I64(v) => Box::new(*v),
                    SqlParam::Bool(v) => Box::new(*v),
                    SqlParam::Text(v) => Box::new(v.clone()),
                    SqlParam::Uuid(v) => Box::new(*v),
                    SqlParam::Decimal(v) => Box::new(v.to_string()),
                    SqlParam::DateTime(v) => Box::new(*v),
                    SqlParam::Bytes(v) => Box::new(v.clone()),
                    SqlParam::Null => Box::new(Option::<i32>::None),
                };
                boxed.push(b);
            }
            let refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
                boxed.iter().map(|b| &**b as _).collect();
            let res = pg.execute(sql, &refs[..]).await?;
            Ok(res)
        }
    }
}
