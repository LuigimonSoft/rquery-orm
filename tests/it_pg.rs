use rquery_orm::{col, connect_postgres, val, Entity, GenericRepository, JoinType, QueryExecutor};

#[derive(Entity, Debug)]
#[table(name = "Employees")]
struct Employee {
    #[key(name = "EmployeeId")]
    pub employee_id: i32,
    #[column(name = "FirstName")]
    pub first_name: String,
    #[column(name = "CountryId")]
    pub country_id: String,
    #[column(name = "HireDate")]
    pub hire_date: chrono::NaiveDateTime,
}

async fn repo() -> anyhow::Result<GenericRepository<Employee>> {
    let db = connect_postgres(
        "localhost",
        5432,
        "tempdb",
        "postgres",
        "YourStrong!Passw0rd",
    )
    .await?;
    Ok(GenericRepository::<Employee>::new(db))
}

#[tokio::test]
#[ignore]
async fn it_pg_select_chain() -> anyhow::Result<()> {
    let repo = repo().await?;

    let list = repo
        .Select()
        .Join(
            JoinType::Left,
            "Countries C",
            col!("Employees.CountryId").eq(col!("C.CountryId")),
        )
        .Where(col!("Employees.CountryId").eq(val!("Mex")))
        .OrderBy("Employees.HireDate DESC")
        .Top(1)
        .ToListAsync()
        .await?;

    assert_eq!(list.len(), 1);
    assert_eq!(list[0].first_name, "Ana");

    Ok(())
}
