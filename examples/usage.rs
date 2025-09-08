use chrono::NaiveDateTime;
use rquery_orm::{col, connect_postgres, on, condition, Entity, GenericRepository, JoinType, QueryExecutor};

mod employees_mod {
    use super::*;
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
        pub country_id: String,
        #[column]
        pub hire_date: NaiveDateTime,
    }
}

mod countries_mod {
    use super::*;
    #[derive(Entity, Debug, Clone)]
    #[table(name = "Countries")]
    pub struct Countries {
        #[key(is_identity = true)]
        pub country_id: String,
        #[column]
        pub name: String,
    }
}

async fn example_simple() -> anyhow::Result<()> {
    use employees_mod::Employees;
    let db = connect_postgres("localhost", 5432, "mydb", "postgres", "secret").await?;
    let employees = GenericRepository::<Employees>::new(db);
    let _rows = employees
        .Select()
        .Join(
            JoinType::Left,
            "Countries C",
            col!("Employees.CountryId").eq(col!("C.CountryId")),
        )
        .Where(condition!(Employees::country_id == "Mex"))
        .OrderBy("Employees.HireDate DESC")
        .Top(10)
        .to_list_async()
        .await?;
    Ok(())
}

async fn example_dual() -> anyhow::Result<()> {
    use countries_mod::Countries;
    use employees_mod::Employees;
    let db = connect_postgres("localhost", 5432, "mydb", "postgres", "secret").await?;
    let repo = GenericRepository::<(Employees, Countries)>::new(db);
    let _pairs = repo
        .Select()
        .Join(JoinType::Left, on!(Employees::country_id == Countries::country_id))
        .Where(condition!(Countries::name == "Mexico"))
        .Top(10)
        .to_list_async()
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Only call functions; these connect to DB but won't run under `cargo test --no-run`.
    let _ = example_simple().await;
    let _ = example_dual().await;
    Ok(())
}
