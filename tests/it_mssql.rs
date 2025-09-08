use chrono::NaiveDate;
use rquery_orm::{
    col, connect_mssql, val, Crud, Entity, GenericRepository, JoinType, QueryExecutor, SqlParam,
};

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
    let db = connect_mssql("localhost", 1433, "tempdb", "sa", "YourStrong!Passw0rd").await?;
    Ok(GenericRepository::<Employee>::new(db))
}

#[tokio::test]
#[ignore]
async fn it_mssql_select() -> anyhow::Result<()> {
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
        .Top(10)
        .to_list_async()
        .await?;

    assert_eq!(list.len(), 1);
    Ok(())
}

#[tokio::test]
#[ignore]
async fn it_mssql_insert() -> anyhow::Result<()> {
    let repo = repo().await?;

    let new_emp = Employee {
        employee_id: 2,
        first_name: "Ana".into(),
        country_id: "USA".into(),
        hire_date: NaiveDate::from_ymd_opt(2022, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
    };
    repo.insert_async(&new_emp).await?;

    let inserted = repo
        .Select()
        .Where(col!("Employees.EmployeeId").eq(val!(2)))
        .to_single_async()
        .await?
        .unwrap();
    assert_eq!(inserted.first_name, "Ana");

    repo.delete_by_key_async(SqlParam::I32(2)).await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn it_mssql_update() -> anyhow::Result<()> {
    let repo = repo().await?;

    let base = Employee {
        employee_id: 3,
        first_name: "Ana".into(),
        country_id: "USA".into(),
        hire_date: NaiveDate::from_ymd_opt(2022, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
    };
    repo.insert_async(&base).await?;

    let upd = Employee {
        first_name: "Ann".into(),
        ..base
    };
    repo.update_async(&upd).await?;

    let updated = repo
        .Select()
        .Where(col!("Employees.EmployeeId").eq(val!(3)))
        .to_single_async()
        .await?
        .unwrap();
    assert_eq!(updated.first_name, "Ann");

    repo.delete_by_key_async(SqlParam::I32(3)).await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn it_mssql_delete() -> anyhow::Result<()> {
    let repo = repo().await?;

    let emp = Employee {
        employee_id: 4,
        first_name: "Bob".into(),
        country_id: "USA".into(),
        hire_date: NaiveDate::from_ymd_opt(2022, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
    };
    repo.insert_async(&emp).await?;

    repo.delete_by_key_async(SqlParam::I32(4)).await?;
    let none = repo
        .Select()
        .Where(col!("Employees.EmployeeId").eq(val!(4)))
        .to_list_async()
        .await?;
    assert_eq!(none.len(), 0);
    Ok(())
}
