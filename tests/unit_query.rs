use rquery_orm::{
    col, val, Entity, FromRowNamed, FromRowWithPrefix, JoinType, Persistable, PlaceholderStyle,
    Query, SqlParam, TableMeta, Validatable,
};

struct Dummy;

impl Entity for Dummy {
    fn table() -> &'static TableMeta {
        &TableMeta {
            name: "dummy",
            schema: None,
            columns: &[],
            keys: &[],
            relations: &[],
        }
    }
}

impl FromRowNamed for Dummy {
    fn from_row_ms(_row: &tiberius::Row) -> anyhow::Result<Self> {
        unimplemented!()
    }
    fn from_row_pg(_row: &tokio_postgres::Row) -> anyhow::Result<Self> {
        unimplemented!()
    }
}

impl FromRowWithPrefix for Dummy {
    fn from_row_ms_with(_row: &tiberius::Row, _prefix: &str) -> anyhow::Result<Self> {
        unimplemented!()
    }
    fn from_row_pg_with(_row: &tokio_postgres::Row, _prefix: &str) -> anyhow::Result<Self> {
        unimplemented!()
    }
}

impl Validatable for Dummy {
    fn validate(&self) -> Result<(), Vec<String>> {
        Ok(())
    }
}

impl Persistable for Dummy {
    fn build_insert(&self, _style: PlaceholderStyle) -> (String, Vec<SqlParam>, bool) {
        unimplemented!()
    }
    fn build_update(&self, _style: PlaceholderStyle) -> (String, Vec<SqlParam>) {
        unimplemented!()
    }
    fn build_delete(&self, _style: PlaceholderStyle) -> (String, Vec<SqlParam>) {
        unimplemented!()
    }
    fn build_delete_by_key(_key: SqlParam, _style: PlaceholderStyle) -> (String, Vec<SqlParam>) {
        unimplemented!()
    }
}

#[test]
fn expr_builds_and_params_order() {
    let e = col!("E.Age")
        .gt(val!(30))
        .and(col!("E.Active").eq(val!(true)));
    let mut ps = vec![];
    let sql = e.to_sql_with(PlaceholderStyle::AtP, &mut ps);
    assert_eq!(sql, "(E.Age > @P1) AND (E.Active = @P2)");
    assert_eq!(ps.len(), 2);
}

#[test]
fn join_and_where_build_for_pg() {
    let q = Query::<Dummy>::new("Employees E", PlaceholderStyle::Dollar)
        .Join(
            JoinType::Left,
            "Countries C",
            col!("E.CountryId").eq(col!("C.CountryId")),
        )
        .Where(condition!("E.CountryId" == "Mex"))
        .OrderBy("E.Id")
        .Top(5);
    let (sql, params) = q.to_sql();
    assert!(sql.contains("LEFT JOIN Countries C ON (E.CountryId = C.CountryId)"));
    assert!(sql.contains("WHERE (E.CountryId = $1)"));
    assert!(sql.ends_with("ORDER BY E.Id LIMIT 5"));
    assert_eq!(params.len(), 1);
}

#[test]
fn full_query_chain_pg() {
    let q = Query::<Dummy>::new("Employees", PlaceholderStyle::Dollar)
        .Join(
            JoinType::Left,
            "Countries C",
            col!("Employees.CountryId").eq(col!("C.CountryId")),
        )
        .Where(condition!("Employees.CountryId" == "Mex"))
        .OrderBy("Employees.HireDate DESC")
        .Top(10);
    let (sql, params) = q.to_sql();
    assert_eq!(
        sql,
        "SELECT * FROM Employees LEFT JOIN Countries C ON (Employees.CountryId = C.CountryId) WHERE (Employees.CountryId = $1) ORDER BY Employees.HireDate DESC LIMIT 10",
    );
    assert_eq!(params, vec![SqlParam::Text("Mex".into())]);
}

#[test]
fn dual_query_builds_sql() {
    // Build a dual query selecting from two entities using typed ON
    let q = rquery_orm::DualQuery::<Dummy, Dummy>::new(PlaceholderStyle::Dollar)
        .Join(JoinType::Left, col!("A").eq(col!("B")))
        .Top(10);
    let (sql, _params) = q.to_sql();
    assert!(sql.contains("SELECT "));
    assert!(sql.contains(" LEFT JOIN "));
}
