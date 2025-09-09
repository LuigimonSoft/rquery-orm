use quote::quote;
use syn::DeriveInput;

fn gen(input: DeriveInput) -> String {
    crate::entity_impl(input).to_string()
}

#[test]
fn generates_entity_impl_and_consts() {
    let input: DeriveInput = syn::parse_quote! {
        #[table(name = "T1", schema = "dbo")]
        struct TestEntity {
            #[key(is_identity = true)]
            id: i32,
            #[column(name = "col_a", required, max_length = 50, min_length = 1, allow_empty, error_required = "e required", error_max_length = "too long", error_min_length = "too short", error_allow_empty = "no empty", error_allow_null = "no null", regex = "^[a-z]+$", error_regex = "bad format", ignore_in_update, ignore_in_insert, ignore_in_delete)]
            a: String,
            #[column]
            b: i32,
            #[column(allow_null = true)]
            c: Option<i32>,
            #[relation(foreign_key = "id", table = "Other", table_number = 2, ignore_in_update, ignore_in_insert)]
            rel: i32,
        }
    };

    let s = gen(input);

    // Core impls
    assert!(s.contains("impl :: rquery_orm :: mapping :: Entity for TestEntity"));
    assert!(s.contains("impl :: rquery_orm :: mapping :: Validatable for TestEntity"));
    assert!(s.contains("impl :: rquery_orm :: mapping :: Persistable for TestEntity"));
    assert!(s.contains("impl :: rquery_orm :: mapping :: FromRowNamed for TestEntity"));
    assert!(s.contains("impl :: rquery_orm :: mapping :: FromRowWithPrefix for TestEntity"));

    // Table meta
    assert!(s.contains("static TABLE_META"));
    assert!(s.contains("name : \"T1\""));
    assert!(s.contains("schema"));

    // Associated consts block
    assert!(s.contains("impl TestEntity { pub const TABLE : & 'static str = \"T1\" ;"));
    assert!(s.contains("pub const id : & 'static str = \"id\" ;"));
    assert!(s.contains("pub const a : & 'static str = \"col_a\" ;"));
}

#[test]
fn builds_sql_fragments() {
    let input: DeriveInput = syn::parse_quote! {
        #[table(name = "T2")]
        struct E2 {
            #[key(is_identity = true)] id: i32,
            #[column] a: i32,
            #[column(ignore_in_update)] b: i32,
            #[column(ignore)] c: i32,
            #[column(ignore_in_insert)] d: i32,
        }
    };
    let s = gen(input);
    // INSERT excludes identity id and ignored fields (format string present)
    assert!(s.contains("INSERT INTO {} ("));
    // UPDATE has SET and WHERE
    assert!(s.contains("UPDATE {} SET"));
    assert!(s.contains("WHERE"));
    // DELETE has WHERE by key
    assert!(s.contains("DELETE FROM {} WHERE"));
}

#[test]
fn validates_string_rules() {
    let input: DeriveInput = syn::parse_quote! {
        struct EV {
            #[key] id: i32,
            #[column(required, min_length = 2, max_length = 4, regex = "^[a-z]+$")] name: String,
        }
    };
    let s = gen(input);
    // Validation branches should appear
    assert!(s.contains("cannot be empty"));
    assert!(s.contains("exceeds max length"));
    assert!(s.contains("below min length"));
    assert!(s.contains("has invalid format"));
}
