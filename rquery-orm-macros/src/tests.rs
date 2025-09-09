use super::*; // use the proc-macro
use crate::rquery_orm::mapping::*;
use crate::rquery_orm::query::*;

#[derive(Entity, Debug, Clone)]
#[table(name = "T1", schema = "dbo")]
struct TestEntity {
    #[key(is_identity = true)]
    id: i32,

    #[column(name = "col_a")]
    a: i32,

    #[column(ignore_in_update)]
    b: i32,

    #[column(ignore)]
    c: i32,

    #[column(ignore_in_insert)]
    d: i32,

    #[column(required, error_required = "e required")]
    e: Option<i32>,

    #[relation(foreign_key = "id", table = "Other", table_number = 2, ignore_in_update, ignore_in_insert)]
    rel: i32,
}

#[test]
fn table_and_columns_metadata() {
    let t = TestEntity::table();
    assert_eq!(t.name, "T1");
    assert_eq!(t.schema, Some("dbo"));
    // Columns exclude relation and respect ignore
    assert!(t.columns.iter().any(|c| c.name == "col_a"));
    assert!(t.columns.iter().any(|c| c.name == "b"));
    assert!(t.columns.iter().any(|c| c.name == "d"));
    assert!(t.columns.iter().any(|c| c.name == "e"));
    assert!(t.columns.iter().any(|c| c.name == "id"));
    assert!(!t.columns.iter().any(|c| c.name == "c"));
    // Keys
    assert_eq!(t.keys.len(), 1);
    assert_eq!(t.keys[0].column, "id");
    assert!(t.keys[0].is_identity);
    // Relations
    assert_eq!(t.relations.len(), 1);
    let r = &t.relations[0];
    assert_eq!(r.name, "rel");
    assert_eq!(r.foreign_key, "id");
    assert_eq!(r.table, "Other");
    assert_eq!(r.table_number, Some(2));
    assert!(r.ignore_in_update);
    assert!(r.ignore_in_insert);
    // Associated consts
    assert_eq!(TestEntity::TABLE, "T1");
    assert_eq!(TestEntity::id, "id");
    assert_eq!(TestEntity::a, "col_a");
    assert_eq!(TestEntity::b, "b");
    assert_eq!(TestEntity::d, "d");
    assert_eq!(TestEntity::e, "e");
}

#[test]
fn validate_required_and_allow_flags() {
    let te = TestEntity { id: 0, a: 1, b: 2, c: 3, d: 4, e: None, rel: 0 };
    let errs = te.validate().unwrap_err();
    assert!(errs.contains(&"e required".to_string()));
}

#[test]
fn build_insert_update_delete_sql() {
    let te = TestEntity { id: 10, a: 11, b: 12, c: 13, d: 14, e: Some(15), rel: 0 };

    // INSERT: exclude identity id, ignore c, and ignore_in_insert d
    let (sql_i, params_i, has_identity) = te.build_insert(PlaceholderStyle::Dollar);
    assert_eq!(sql_i, "INSERT INTO T1 (col_a, b, e) VALUES ($1, $2, $3)");
    assert_eq!(params_i.len(), 3);
    assert!(has_identity);

    // UPDATE: set a, d, e; where id; b is ignore_in_update; c ignored
    let (sql_u, params_u) = te.build_update(PlaceholderStyle::AtP);
    assert_eq!(sql_u, "UPDATE T1 SET col_a = @P1, d = @P2, e = @P3 WHERE id = @P4");
    assert_eq!(params_u.len(), 4);

    // DELETE: where id
    let (sql_d, params_d) = te.build_delete(PlaceholderStyle::Dollar);
    assert_eq!(sql_d, "DELETE FROM T1 WHERE id = $1");
    assert_eq!(params_d.len(), 1);

    // Static delete by key uses first key column
    let (sql_dbk, params_dbk) = TestEntity::build_delete_by_key(SqlParam::Int(99), PlaceholderStyle::Dollar);
    assert_eq!(sql_dbk, "DELETE FROM T1 WHERE id = $1");
    assert_eq!(params_dbk.len(), 1);
}

#[test]
fn key_trait_impl() {
    let te = TestEntity { id: 7, a: 0, b: 0, c: 0, d: 0, e: None, rel: 0 };
    use crate::rquery_orm::mapping::KeyAsInt;
    assert_eq!(te.key(), 7);
}

