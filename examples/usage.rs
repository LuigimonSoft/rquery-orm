use chrono::NaiveDateTime;
use rquery_orm::{col, connect_postgres, val, Entity, GenericRepository, JoinType, QueryExecutor};

#[derive(Entity, Debug, Clone)]
#[table(name = "Employees")]
pub struct Employees {
    #[key(is_identity = true)]
    pub employee_id: i32,
    #[column]
    pub first_name: String,
    #[column]
    pub last_name: String,
    #[column]
    pub age: i32,
    #[column]
    pub hire_date: NaiveDateTime,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db = connect_postgres("localhost", 5432, "mydb", "postgres", "secret").await?;
    let employees = GenericRepository::<Employees>::new(db);

    let list = employees
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

    println!("rows: {}", list.len());
    Ok(())
}
