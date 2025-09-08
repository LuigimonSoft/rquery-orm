use std::marker::PhantomData;

use std::sync::Arc;

use crate::db::DatabaseRef;
use crate::mapping::Entity;
use anyhow::Result;
use futures::TryStreamExt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceholderStyle {
    AtP,
    Dollar,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SqlParam {
    I32(i32),
    I64(i64),
    Bool(bool),
    Text(String),
    Uuid(uuid::Uuid),
    Decimal(rust_decimal::Decimal),
    DateTime(chrono::NaiveDateTime),
    Bytes(Vec<u8>),
    Null,
}

pub trait ToParam {
    fn to_param(self) -> SqlParam;
}

impl ToParam for i32 {
    fn to_param(self) -> SqlParam {
        SqlParam::I32(self)
    }
}
impl ToParam for i64 {
    fn to_param(self) -> SqlParam {
        SqlParam::I64(self)
    }
}
impl ToParam for bool {
    fn to_param(self) -> SqlParam {
        SqlParam::Bool(self)
    }
}
impl ToParam for String {
    fn to_param(self) -> SqlParam {
        SqlParam::Text(self)
    }
}
impl<'a> ToParam for &'a str {
    fn to_param(self) -> SqlParam {
        SqlParam::Text(self.to_string())
    }
}
impl ToParam for uuid::Uuid {
    fn to_param(self) -> SqlParam {
        SqlParam::Uuid(self)
    }
}
impl ToParam for rust_decimal::Decimal {
    fn to_param(self) -> SqlParam {
        SqlParam::Decimal(self)
    }
}
impl ToParam for chrono::NaiveDateTime {
    fn to_param(self) -> SqlParam {
        SqlParam::DateTime(self)
    }
}
impl ToParam for Vec<u8> {
    fn to_param(self) -> SqlParam {
        SqlParam::Bytes(self)
    }
}

impl<T: ToParam> ToParam for Option<T> {
    fn to_param(self) -> SqlParam {
        match self {
            Some(v) => v.to_param(),
            None => SqlParam::Null,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    Col(String),
    Param(SqlParam),
    Binary {
        left: Box<Expr>,
        op: &'static str,
        right: Box<Expr>,
    },
    Like {
        left: Box<Expr>,
        right: SqlParam,
    },
    InList {
        left: Box<Expr>,
        list: Vec<SqlParam>,
    },
    Group(Box<Expr>),
}

impl Expr {
    pub fn eq(self, rhs: Expr) -> Expr {
        Expr::Binary {
            left: Box::new(self),
            op: "=",
            right: Box::new(rhs),
        }
    }
    pub fn ne(self, rhs: Expr) -> Expr {
        Expr::Binary {
            left: Box::new(self),
            op: "<>",
            right: Box::new(rhs),
        }
    }
    pub fn gt(self, rhs: Expr) -> Expr {
        Expr::Binary {
            left: Box::new(self),
            op: ">",
            right: Box::new(rhs),
        }
    }
    pub fn ge(self, rhs: Expr) -> Expr {
        Expr::Binary {
            left: Box::new(self),
            op: ">=",
            right: Box::new(rhs),
        }
    }
    pub fn lt(self, rhs: Expr) -> Expr {
        Expr::Binary {
            left: Box::new(self),
            op: "<",
            right: Box::new(rhs),
        }
    }
    pub fn le(self, rhs: Expr) -> Expr {
        Expr::Binary {
            left: Box::new(self),
            op: "<=",
            right: Box::new(rhs),
        }
    }
    pub fn and(self, rhs: Expr) -> Expr {
        Expr::Binary {
            left: Box::new(self),
            op: "AND",
            right: Box::new(rhs),
        }
    }
    pub fn or(self, rhs: Expr) -> Expr {
        Expr::Binary {
            left: Box::new(self),
            op: "OR",
            right: Box::new(rhs),
        }
    }
    pub fn like(self, pattern: Expr) -> Expr {
        match pattern {
            Expr::Param(p) => Expr::Like {
                left: Box::new(self),
                right: p,
            },
            other => panic!("like expects Expr::Param but received {:?}", other),
        }
    }
    pub fn in_list(self, list: Vec<Expr>) -> Expr {
        let mut ps = Vec::new();
        for e in list {
            match e {
                Expr::Param(p) => ps.push(p),
                other => panic!("in_list expects Expr::Param items but received {:?}", other),
            }
        }
        Expr::InList {
            left: Box::new(self),
            list: ps,
        }
    }
    pub fn group(self) -> Expr {
        Expr::Group(Box::new(self))
    }

    pub fn to_sql_with(&self, style: PlaceholderStyle, params: &mut Vec<SqlParam>) -> String {
        match self {
            Expr::Col(c) => c.clone(),
            Expr::Param(p) => {
                params.push(p.clone());
                let idx = params.len();
                match style {
                    PlaceholderStyle::AtP => format!("@P{}", idx),
                    PlaceholderStyle::Dollar => format!("${}", idx),
                }
            }
            Expr::Binary { left, op, right } => {
                let l = left.to_sql_with(style, params);
                let r = right.to_sql_with(style, params);
                if *op == "AND" || *op == "OR" {
                    format!("{} {} {}", l, op, r)
                } else {
                    format!("({} {} {})", l, op, r)
                }
            }
            Expr::Like { left, right } => {
                params.push(right.clone());
                let idx = params.len();
                let ph = match style {
                    PlaceholderStyle::AtP => format!("@P{}", idx),
                    PlaceholderStyle::Dollar => format!("${}", idx),
                };
                format!("({} LIKE {})", left.to_sql_with(style, params), ph)
            }
            Expr::InList { left, list } => {
                let mut phs = Vec::new();
                for p in list {
                    params.push(p.clone());
                    let idx = params.len();
                    phs.push(match style {
                        PlaceholderStyle::AtP => format!("@P{}", idx),
                        PlaceholderStyle::Dollar => format!("${}", idx),
                    });
                }
                format!(
                    "{} IN ({})",
                    left.to_sql_with(style, params),
                    phs.join(", ")
                )
            }
            Expr::Group(e) => format!("({})", e.to_sql_with(style, params)),
        }
    }
}

#[macro_export]
macro_rules! col {
    ($name:expr) => {
        $crate::query::Expr::Col($name.to_string())
    };
}

#[macro_export]
macro_rules! val {
    ($v:expr) => {
        $crate::query::Expr::Param($crate::query::ToParam::to_param($v))
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

impl JoinType {
    fn to_sql(self) -> &'static str {
        match self {
            JoinType::Inner => "INNER JOIN",
            JoinType::Left => "LEFT JOIN",
            JoinType::Right => "RIGHT JOIN",
            JoinType::Full => "FULL JOIN",
        }
    }
}

struct JoinClause {
    join_type: JoinType,
    table: String,
    on: Expr,
}

#[allow(non_snake_case)]
pub struct Query<T>
where
    T: Entity + crate::mapping::FromRowNamed,
{
    table: String,
    style: PlaceholderStyle,
    db: Option<Arc<DatabaseRef>>,
    joins: Vec<JoinClause>,
    filters: Vec<Expr>,
    order_by: Option<String>,
    top: Option<i64>,
    _t: PhantomData<T>,
}

#[allow(non_snake_case)]
impl<T> Query<T>
where
    T: Entity + crate::mapping::FromRowNamed,
{
    pub fn new(table: &str, style: PlaceholderStyle) -> Self {
        Self {
            table: table.to_string(),
            style,
            db: None,
            joins: Vec::new(),
            filters: Vec::new(),
            order_by: None,
            top: None,
            _t: PhantomData,
        }
    }

    pub fn with_db(mut self, db: Arc<DatabaseRef>) -> Self {
        self.db = Some(db);
        self
    }

    pub fn Where(mut self, expr: Expr) -> Self {
        self.filters.push(expr);
        self
    }

    pub fn Join(mut self, join_type: JoinType, table: &str, on_expr: Expr) -> Self {
        self.joins.push(JoinClause {
            join_type,
            table: table.to_string(),
            on: on_expr,
        });
        self
    }

    pub fn OrderBy(mut self, ob: &str) -> Self {
        self.order_by = Some(ob.to_string());
        self
    }

    pub fn Top(mut self, n: i64) -> Self {
        self.top = Some(n);
        self
    }

    pub fn to_sql(&self) -> (String, Vec<SqlParam>) {
        let mut params = Vec::new();
        let mut sql = String::new();
        match self.style {
            PlaceholderStyle::AtP => {
                if let Some(n) = self.top {
                    sql.push_str(&format!("SELECT TOP({}) * FROM {}", n, self.table));
                } else {
                    sql.push_str(&format!("SELECT * FROM {}", self.table));
                }
            }
            PlaceholderStyle::Dollar => {
                sql.push_str(&format!("SELECT * FROM {}", self.table));
            }
        }
        for j in &self.joins {
            sql.push(' ');
            sql.push_str(j.join_type.to_sql());
            sql.push(' ');
            sql.push_str(&j.table);
            sql.push_str(" ON ");
            sql.push_str(&j.on.to_sql_with(self.style, &mut params));
        }
        if !self.filters.is_empty() {
            let mut it = self.filters.iter();
            if let Some(first) = it.next() {
                sql.push_str(" WHERE ");
                sql.push_str(&first.to_sql_with(self.style, &mut params));
                for f in it {
                    sql.push_str(" AND ");
                    sql.push_str(&f.to_sql_with(self.style, &mut params));
                }
            }
        }
        if let Some(ob) = &self.order_by {
            sql.push_str(" ORDER BY ");
            sql.push_str(ob);
        }
        if let Some(n) = self.top {
            if self.style == PlaceholderStyle::Dollar {
                sql.push_str(&format!(" LIMIT {}", n));
            }
        }
        (sql, params)
    }

    pub async fn to_list_async(self) -> Result<Vec<T>> {
        let db = self.db.clone().expect("database reference not set");
        let (sql, params) = self.to_sql();
        match db.as_ref() {
            DatabaseRef::Mssql(conn) => {
                let mut guard = conn.lock().await;
                let mut boxed: Vec<Box<dyn tiberius::ToSql + Send + Sync>> = Vec::new();
                for p in &params {
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
                let mut stream = guard.query(&sql, &refs[..]).await?;
                let mut out = Vec::new();
                while let Some(item) = stream.try_next().await? {
                    if let Some(row) = item.into_row() {
                        out.push(T::from_row_ms(&row)?);
                    }
                }
                Ok(out)
            }
            DatabaseRef::Postgres(pg) => {
                let mut boxed: Vec<Box<dyn tokio_postgres::types::ToSql + Send + Sync>> =
                    Vec::new();
                for p in &params {
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
                let rows = pg.query(&sql, &refs[..]).await?;
                let mut out = Vec::new();
                for row in rows {
                    out.push(T::from_row_pg(&row)?);
                }
                Ok(out)
            }
        }
    }

    pub async fn to_single_async(self) -> Result<Option<T>> {
        let mut list = self.Top(1).to_list_async().await?;
        Ok(list.pop())
    }

    pub async fn ToDictionaryKeyIntAsync(
        self,
    ) -> anyhow::Result<std::collections::HashMap<i32, T>> {
        unimplemented!("execution not implemented");
    }

    pub async fn ToDictionaryKeyGuidAsync(
        self,
    ) -> anyhow::Result<std::collections::HashMap<uuid::Uuid, T>> {
        unimplemented!("execution not implemented");
    }

    pub async fn ToDictionaryKeyStringAsync(
        self,
    ) -> anyhow::Result<std::collections::HashMap<String, T>> {
        unimplemented!("execution not implemented");
    }
}
