use crate::mapping::{Entity, FromRowNamed, Persistable, Validatable};
use crate::{col, val, Repository};

pub struct GenericListService<R, T>
where
    R: Repository<T>,
    T: Entity + FromRowNamed + Validatable + Persistable + Send + Sync,
{
    repo: R,
    _t: std::marker::PhantomData<T>,
}

impl<R, T> GenericListService<R, T>
where
    R: Repository<T>,
    T: Entity + FromRowNamed + Validatable + Persistable + Send + Sync,
{
    pub fn new(repo: R) -> Self {
        Self {
            repo,
            _t: Default::default(),
        }
    }

    pub async fn list_by_country(&self, country: &str) -> anyhow::Result<Vec<T>> {
        let q = self
            .repo
            .Select()
            .Where(col!("E.CountryId").eq(val!(country)));
        q.ToListAsync().await
    }
}
