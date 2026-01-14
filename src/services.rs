use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{AiAudit, CreateAuditRequest};

pub async fn list_audits(pool: &PgPool) -> Result<Vec<AiAudit>, sqlx::Error> {
    sqlx::query_as::<_, AiAudit>(
        "SELECT id, prompt, codigo_generado, es_valido, error_compilacion, created_at FROM ai_audits ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await
}

pub async fn get_audit_by_id(pool: &PgPool, id: Uuid) -> Result<Option<AiAudit>, sqlx::Error> {
    sqlx::query_as::<_, AiAudit>(
        "SELECT id, prompt, codigo_generado, es_valido, error_compilacion, created_at FROM ai_audits WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn create_audit(pool: &PgPool, input: &CreateAuditRequest) -> Result<AiAudit, sqlx::Error> {
    sqlx::query_as::<_, AiAudit>(
        r#"
        INSERT INTO ai_audits (prompt, codigo_generado, es_valido, error_compilacion)
        VALUES ($1, $2, $3, $4)
        RETURNING id, prompt, codigo_generado, es_valido, error_compilacion, created_at
        "#,
    )
    .bind(&input.prompt)
    .bind(&input.codigo_generado)
    .bind(input.es_valido)
    .bind(&input.error_compilacion)
    .fetch_one(pool)
    .await
}
