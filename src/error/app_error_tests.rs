use super::app_error::*;

#[test]
fn displays_not_found_message() {
    let err = AppError::NotFound("missing".to_string());
    assert_eq!(err.to_string(), "missing");
}

#[test]
fn displays_missing_env_var() {
    let err = AppError::MissingEnvVar("STORAGE_URL");
    assert_eq!(err.to_string(), "Missing environment variable: STORAGE_URL");
}

#[test]
fn displays_database_error() {
    #[cfg(feature = "database")]
    {
        let err = AppError::DatabaseError(sqlx::Error::RowNotFound);
        assert!(err.to_string().contains("Database error"));
    }
}
