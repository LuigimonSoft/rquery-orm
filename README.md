# rquery-orm

Lightweight ORM for Rust providing a SQL style query builder over SQL Server and PostgreSQL. It exposes a small set of traits and a derive macro so your structs become database entities, while all SQL is generated through a typed DSL.

## Connecting to the database
```rust
use rquery_orm::connect_postgres;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // PostgreSQL connection
    let db = connect_postgres("localhost", 5432, "mydb", "postgres", "secret").await?;
    Ok(())
}
```

For SQL Server use `connect_mssql` instead.

## Declaring an entity
Annotate your struct with `#[derive(Entity)]` and mark each column:
```rust
use chrono::NaiveDateTime;
use rquery_orm::Entity;

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
```

## Building queries
All queries start from a `GenericRepository` tied to an entity. Chaining methods configures the SQL without executing it until an async terminal call:
```rust
use rquery_orm::{col, val, GenericRepository, JoinType, QueryExecutor};

let repo = GenericRepository::<Employees>::new(db);
let rows = repo
    .Select()
    .Join(JoinType::Left, "Countries C", col!("Employees.CountryId").eq(col!("C.CountryId")))
    .Where(col!("Employees.CountryId").eq(val!("Mex")))
    .OrderBy("Employees.HireDate DESC")
    .Top(10)
    .to_list_async()
    .await?;
```

### Insert
```rust
let employee = Employees { employee_id: 0, first_name: "Ann".into(), last_name: "Lee".into(), age: 30, hire_date: chrono::Utc::now().naive_utc() };
repo.insert_async(&employee).await?;
```

### Update
```rust
let mut e = rows[0].clone();
e.last_name = "Updated".into();
repo.update_async(&e).await?;
```

### Delete
```rust
repo.delete_by_key_async(val!(e.employee_id)).await?;
```

## Validations
Columns can declare validation rules so `validate()` and CRUD operations fail fast before reaching the database:
```rust
use rquery_orm::Entity;

#[derive(Entity)]
#[table(name = "Users")]
pub struct User {
    #[key(is_identity = true)]
    pub id: i32,
    #[column(required, max_length = 30, error_required = "Username is required", error_max_length = "Max 30 chars")]
    pub username: String,
    #[column(regex = "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$", error_regex = "Invalid email format")]
    pub email: String,
}

let user = User { id: 0, username: "".into(), email: "bad".into() };
if let Err(errors) = user.validate() {
    for e in errors { println!("{e}"); }
}
```

When using repository methods like `insert_async` or `update_async`, validation runs automatically; failed validation aborts the operation.

---
See `examples/usage.rs` and the tests folder for additional scenarios.
