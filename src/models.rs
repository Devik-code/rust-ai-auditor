use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct AiAudit {
    pub id: Uuid,
    pub prompt: String,
    pub codigo_generado: String,
    pub es_valido: bool,
    pub error_compilacion: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAuditRequest {
    pub prompt: String,
    pub codigo_generado: String,
    pub es_valido: bool,
    pub error_compilacion: Option<String>,
}
