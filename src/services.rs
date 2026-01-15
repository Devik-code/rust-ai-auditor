use std::process::Command;

use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{AiAudit, CreateAuditRequest};

/// Validates Rust code and returns (is_valid, error_message)
pub fn validate_code(codigo: &str) -> (bool, Option<String>) {
    let mut errors: Vec<String> = Vec::new();

    // Check for balanced braces
    let open_braces = codigo.matches('{').count();
    let close_braces = codigo.matches('}').count();
    if open_braces != close_braces {
        errors.push(format!(
            "Llaves desbalanceadas: {} abiertas, {} cerradas",
            open_braces, close_braces
        ));
    }

    // Check for balanced parentheses
    let open_parens = codigo.matches('(').count();
    let close_parens = codigo.matches(')').count();
    if open_parens != close_parens {
        errors.push(format!(
            "Paréntesis desbalanceados: {} abiertos, {} cerrados",
            open_parens, close_parens
        ));
    }

    // Check for balanced brackets
    let open_brackets = codigo.matches('[').count();
    let close_brackets = codigo.matches(']').count();
    if open_brackets != close_brackets {
        errors.push(format!(
            "Corchetes desbalanceados: {} abiertos, {} cerrados",
            open_brackets, close_brackets
        ));
    }

    // Check for fn main if it looks like a complete program
    let has_fn = codigo.contains("fn ");
    let has_main = codigo.contains("fn main");
    if has_fn && !has_main && !codigo.contains("pub fn") && !codigo.contains("impl ") {
        // Looks like standalone functions without main
        errors.push("Posible falta de función main() para un programa ejecutable".to_string());
    }

    // Check for prohibited/dangerous patterns
    let prohibited_patterns = [
        ("std::process::Command", "Uso de Command puede ser peligroso"),
        ("std::fs::remove", "Operación de eliminación de archivos detectada"),
        ("unsafe {", "Bloque unsafe detectado - requiere revisión manual"),
    ];

    for (pattern, message) in prohibited_patterns {
        if codigo.contains(pattern) {
            errors.push(message.to_string());
        }
    }

    // Check for common syntax errors
    if codigo.contains("let ") && !codigo.contains(';') {
        errors.push("Posible falta de punto y coma en declaración let".to_string());
    }

    // Check for empty function bodies
    if codigo.contains("fn ") && codigo.contains("{}") {
        errors.push("Función con cuerpo vacío detectada".to_string());
    }

    if errors.is_empty() {
        (true, None)
    } else {
        let error_message = errors.join("; ");
        tracing::warn!(errors = %error_message, "Código marcado como inválido");
        (false, Some(error_message))
    }
}

/// Mock function to test process execution permissions
/// Executes `rustc --version` to verify the server can run processes
#[allow(dead_code)]
pub fn check_compilation(_code: &str) -> Result<(), String> {
    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .map_err(|e| format!("Failed to execute rustc: {}", e))?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        tracing::info!(rustc_version = %version.trim(), "rustc disponible en el sistema");
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(format!("rustc failed: {}", error))
    }
}

#[tracing::instrument(skip(pool))]
pub async fn list_audits(pool: &PgPool) -> Result<Vec<AiAudit>, sqlx::Error> {
    sqlx::query_as::<_, AiAudit>(
        "SELECT id, prompt, codigo_generado, es_valido, error_compilacion, created_at FROM ai_audits ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await
}

#[tracing::instrument(skip(pool))]
pub async fn get_audit_by_id(pool: &PgPool, id: Uuid) -> Result<Option<AiAudit>, sqlx::Error> {
    sqlx::query_as::<_, AiAudit>(
        "SELECT id, prompt, codigo_generado, es_valido, error_compilacion, created_at FROM ai_audits WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

#[tracing::instrument(skip(pool))]
pub async fn create_audit(pool: &PgPool, input: &CreateAuditRequest) -> Result<AiAudit, sqlx::Error> {
    // Validate code before inserting
    let (es_valido, error_compilacion) = validate_code(&input.codigo_generado);

    sqlx::query_as::<_, AiAudit>(
        r#"
        INSERT INTO ai_audits (prompt, codigo_generado, es_valido, error_compilacion)
        VALUES ($1, $2, $3, $4)
        RETURNING id, prompt, codigo_generado, es_valido, error_compilacion, created_at
        "#,
    )
    .bind(&input.prompt)
    .bind(&input.codigo_generado)
    .bind(es_valido)
    .bind(&error_compilacion)
    .fetch_one(pool)
    .await
}
