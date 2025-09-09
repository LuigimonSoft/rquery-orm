# rquery-orm-macros

Procedural macros for rquery-orm. It exposes the `#[derive(Entity)]` derive and
its attributes to turn Rust structs into SQL-mapped entities. The main crate
`rquery-orm` re-exports this derive, so in most cases you can just `use rquery_orm::Entity;`.

- Supported backends: SQL Server (tiberius) and PostgreSQL (tokio-postgres)
- Generates table/column metadata, validations, and SQL helpers

For an overview of the ORM and full examples, check `../README.md` or the
`rquery-orm` crate page.

## Quick start

```rust
use rquery_orm::Entity; // re-exported from rquery-orm

#[derive(Entity, Debug, Clone)]
#[table(name = "Employees", schema = "dbo")] // schema is optional
pub struct Employees {
    #[key(is_identity = true)]
    pub employee_id: i32,

    #[column(required, max_length = 50)]
    pub first_name: String,

    #[column]
    pub last_name: String,
}
```

The derive implements traits used by `rquery-orm`:
- `Entity`: access to table metadata
- `FromRowNamed` and `FromRowWithPrefix`: mapping from `tiberius::Row` and `tokio_postgres::Row`
- `Validatable`: attribute-based data validation
- `Persistable`: builds SQL for `INSERT`, `UPDATE`, and `DELETE`

It also generates associated constants:
- `YourType::TABLE` with the table name
- For each field, a constant with the column name (e.g. `Employees::first_name`)

If the first key is `i32`, `String`, or `uuid::Uuid`, it implements `KeyAsInt`,
`KeyAsString`, or `KeyAsGuid` to expose `self.key()`.

## Available attributes

- `#[table(...)]`
  - `name = "..."`: table name (defaults to the struct name)
  - `schema = "..."`: schema name (optional)

- `#[column(...)]` (for non-relation fields)
  - Presence only: `#[column]`
  - Validation:
    - `required`: value must be present (not `None`) and, for `String`, non-empty
    - `allow_null = true|false`: allow `None` on `Option<T>` (defaults to `false`)
    - `allow_empty`: allow empty string for `String` (allowed by default)
    - `max_length = N`, `min_length = N`
    - `regex = "..."`: pattern for `String`
    - Custom error messages: `error_required`, `error_allow_null`,
      `error_allow_empty`, `error_max_length`, `error_min_length`, `error_regex`
  - Ignore in operations:
    - `ignore`: ignore the column in all operations
    - `ignore_in_update`, `ignore_in_insert`, `ignore_in_delete`
  - `name = "..."`: column name when it differs from the field

- `#[key(...)]` (on key fields)
  - `is_identity = true|false`: identity/serial column (omitted from INSERT)
  - `name = "..."`: column name if different
  - `ignore_in_update`, `ignore_in_insert`: fine-grained control per operation

- `#[relation(...)]` (on relation fields, metadata only)
  - `foreign_key = "..."`: FK name in the current entity
  - `table = "..."`: related table
  - `table_number = N`: logical index/alias (optional)
  - `ignore_in_update`, `ignore_in_insert`

## Generated SQL and placeholders

`Persistable` methods construct SQL and parameters using the placeholder style
selected by `rquery-orm`:
- `PlaceholderStyle::AtP` → `@P1, @P2, ...` (SQL Server)
- `PlaceholderStyle::Dollar` → `$1, $2, ...` (PostgreSQL)

Example (conceptual):
```rust
use rquery_orm::PlaceholderStyle;
let (sql, params, has_identity) = entity.build_insert(PlaceholderStyle::Dollar);
```

## Installation

Typically you consume the derive re-exported from the main crate:

```toml
[dependencies]
rquery-orm = "1.0.0"
```

If you really need the macros crate directly (not recommended), add it explicitly:

```toml
[dependencies]
rquery-orm-macros = "1.0.0"
```

## License

MIT © Luis Carlos Carrillo
