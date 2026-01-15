//! Defines the GraphQL schema, including queries and mutations.

use crate::{
    error::AppError,
    models::{AiAudit, AuditStats, CreateAuditRequest},
    services,
};
use async_graphql::{Context, Object, Schema};
use sqlx::PgPool;
use uuid::Uuid;

/// The root of all GraphQL queries.
#[derive(Default)]
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Retrieves a list of all AI audits, sorted by creation date.
    async fn audits(&self, ctx: &Context<'_>) -> Result<Vec<AiAudit>, AppError> {
        let pool = ctx
            .data::<PgPool>()
            .map_err(|_| AppError::NotFound("Database pool not found in context".to_string()))?;
        services::list_audits(pool).await
    }

    /// Retrieves a single AI audit by its unique identifier.
    async fn audit(&self, ctx: &Context<'_>, id: Uuid) -> Result<Option<AiAudit>, AppError> {
        let pool = ctx
            .data::<PgPool>()
            .map_err(|_| AppError::NotFound("Database pool not found in context".to_string()))?;
        services::get_audit_by_id(pool, id).await
    }

    /// Retrieves aggregated statistics about all audits.
    async fn stats(&self, ctx: &Context<'_>) -> Result<AuditStats, AppError> {
        let pool = ctx
            .data::<PgPool>()
            .map_err(|_| AppError::NotFound("Database pool not found in context".to_string()))?;
        services::get_audit_stats(pool).await
    }
}

/// The root of all GraphQL mutations.
#[derive(Default)]
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Creates a new AI audit.
    ///
    /// It takes a prompt and the AI-generated code as input, performs a compilation check,
    /// and stores the result in the database.
    async fn create_audit(
        &self,
        ctx: &Context<'_>,
        input: CreateAuditRequest,
    ) -> Result<AiAudit, AppError> {
        let pool = ctx
            .data::<PgPool>()
            .map_err(|_| AppError::NotFound("Database pool not found in context".to_string()))?;
        services::create_audit(pool, &input).await
    }
}

/// The application's complete GraphQL schema.
pub type AppSchema = Schema<QueryRoot, MutationRoot, async_graphql::EmptySubscription>;
