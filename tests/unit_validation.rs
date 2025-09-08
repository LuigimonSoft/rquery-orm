use rquery_orm::{Entity, Validatable};

#[derive(Entity, Debug)]
#[table(name = "Users")]
struct User {
    #[key(is_identity = true)]
    id: i32,

    #[column(
        required,
        max_length = 30,
        error_required = "Username is required",
        error_max_length = "Max 30 chars"
    )]
    username: String,

    #[column(
        required,
        regex = "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$",
        error_regex = "Invalid email format"
    )]
    email: String,

    #[column(min_length = 8, error_min_length = "Password must be at least 8 chars")]
    password: String,

    #[column(allow_null = true)]
    bio: Option<String>,
}

#[test]
fn validation_fails() {
    let user = User {
        id: 0,
        username: "".into(),
        email: "invalid".into(),
        password: "123".into(),
        bio: None,
    };
    let errs = user.validate().unwrap_err();
    assert!(errs.contains(&"Username is required".to_string()));
    assert!(errs.contains(&"Invalid email format".to_string()));
    assert!(errs.contains(&"Password must be at least 8 chars".to_string()));
}

#[test]
fn validation_passes() {
    let user = User {
        id: 1,
        username: "john".into(),
        email: "john@example.com".into(),
        password: "password123".into(),
        bio: None,
    };
    assert!(user.validate().is_ok());
}
