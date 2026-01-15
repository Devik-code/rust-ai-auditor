//! Contains the core business logic for database operations.

use crate::{
    auditor,
    error::AppError,
    models::{AiAudit, AuditStats, CommonError, CreateAuditRequest},
};
use sqlx::PgPool;
use uuid::Uuid;

/// Retrieves a list of all AI audits from the database, sorted by creation date.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
///
/// # Returns
///
/// * `Ok(Vec<AiAudit>)` - A vector of audit records.
/// * `Err(AppError::Sqlx)` - If a database query fails.
#[tracing::instrument(skip(pool))]
pub async fn list_audits(pool: &PgPool) -> Result<Vec<AiAudit>, AppError> {
    sqlx::query_as::<_, AiAudit>(
        "SELECT id, prompt, generated_code, is_valid, compilation_error, created_at FROM ai_audits ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::from)
}

/// Retrieves a single AI audit by its ID.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
/// * `id` - The UUID of the audit to retrieve.
///
/// # Returns
///
/// * `Ok(Option<AiAudit>)` - The audit record if found, otherwise `None`.
/// * `Err(AppError::Sqlx)` - If a database query fails.
#[tracing::instrument(skip(pool))]
pub async fn get_audit_by_id(pool: &PgPool, id: Uuid) -> Result<Option<AiAudit>, AppError> {
    sqlx::query_as::<_, AiAudit>(
        "SELECT id, prompt, generated_code, is_valid, compilation_error, created_at FROM ai_audits WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::from)
}

/// Creates a new AI audit record in the database.
///
/// This function first compiles the provided code using `auditor::check_compilation`.
/// Based on the result, it sets the `is_valid` and `compilation_error` fields
/// before inserting the new record into the database.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
/// * `input` - The request payload containing the prompt and generated code.
///
/// # Returns
///
/// * `Ok(AiAudit)` - The newly created audit record.
/// * `Err(AppError)` - If the code compilation or database insertion fails.
#[tracing::instrument(skip(pool, input))]
pub async fn create_audit(pool: &PgPool, input: &CreateAuditRequest) -> Result<AiAudit, AppError> {
    // Compile the generated code to determine its validity.
    let (is_valid, compilation_error) = match auditor::check_compilation(&input.generated_code) {
        Ok(()) => (true, None),
        Err(AppError::Audit(e)) => (false, Some(e)),
        Err(e) => return Err(e), // Propagate other error types
    };

    sqlx::query_as::<_, AiAudit>(
        r#"
        INSERT INTO ai_audits (prompt, generated_code, is_valid, compilation_error)
        VALUES ($1, $2, $3, $4)
        RETURNING id, prompt, generated_code, is_valid, compilation_error, created_at
        "#,
    )
    .bind(&input.prompt)
    .bind(&input.generated_code)
    .bind(is_valid)
    .bind(compilation_error)
    .fetch_one(pool)
    .await
    .map_err(AppError::from)
}

/// Calculates and retrieves statistics about all AI audits.
///
/// # Arguments
///
/// * `pool` - A reference to the database connection pool.
///
/// # Returns
///
/// * `Ok(AuditStats)` - The calculated statistics.
/// * `Err(AppError::Sqlx)` - If any database query fails.
#[tracing::instrument(skip(pool))]
pub async fn get_audit_stats(pool: &PgPool) -> Result<AuditStats, AppError> {
    // Get total and valid counts.
    let (total_audits, valid_audits): (i64, i64) = sqlx::query_as(
        "SELECT
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE is_valid = true) as valid
         FROM ai_audits",
    )
    .fetch_one(pool)
    .await?;

    let invalid_audits = total_audits - valid_audits;
    let validation_rate = if total_audits > 0 {
        valid_audits as f64 / total_audits as f64
    } else {
        0.0
    };

    // Get the most common compilation errors.
    let common_errors = sqlx::query_as::<_, CommonError>(
        r#"
        SELECT
            LEFT(compilation_error, 200) as error_message,
            COUNT(*) as frequency
        FROM ai_audits
        WHERE compilation_error IS NOT NULL
          AND compilation_error != ''
        GROUP BY LEFT(compilation_error, 200)
        ORDER BY frequency DESC
        LIMIT 10
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(AuditStats {
        total_audits,
        valid_audits,
        invalid_audits,
        validation_rate,
        common_errors,
    })
}
