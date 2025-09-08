use chrono::{NaiveDate, NaiveDateTime};
use rquery_orm::{Entity, Persistable, PlaceholderStyle};

#[derive(Entity, Debug, Clone)]
#[table(name = "Employees")]
struct Employees {
    #[key(is_identity = true)]
    employee_id: i32,
    #[column]
    first_name: String,
    #[column]
    last_name: String,
    #[column]
    age: i32,
    #[column]
    hire_date: NaiveDateTime,
}

#[test]
fn entity_metadata() {
    let t = Employees::table();
    assert_eq!(t.name, "Employees");
    assert_eq!(t.columns.len(), 5);
    assert_eq!(t.keys.len(), 1);
    assert_eq!(t.keys[0].column, "employee_id");
}

#[test]
fn insert_sql_builds() {
    let emp = Employees {
        employee_id: 0,
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        age: 30,
        hire_date: NaiveDate::from_ymd_opt(1970, 1, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap(),
    };
    let (sql, params, has_id) = emp.build_insert(PlaceholderStyle::Dollar);
    assert_eq!(
        sql,
        "INSERT INTO Employees (first_name, last_name, age, hire_date) VALUES ($1, $2, $3, $4)"
    );
    assert_eq!(params.len(), 4);
    assert!(has_id);
}
