use async_graphql::{Context, Object, Result};
use sqlx::PgPool;

use crate::models::{AiAudit, CreateAuditRequest};
use crate::services;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn audits(&self, ctx: &Context<'_>) -> Result<Vec<AiAudit>> {
        let pool = ctx.data::<PgPool>()?;
        let audits = services::list_audits(pool).await?;
        Ok(audits)
    }

    async fn audit(&self, ctx: &Context<'_>, id: uuid::Uuid) -> Result<Option<AiAudit>> {
        let pool = ctx.data::<PgPool>()?;
        let audit = services::get_audit_by_id(pool, id).await?;
        Ok(audit)
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_audit(&self, ctx: &Context<'_>, input: CreateAuditRequest) -> Result<AiAudit> {
        let pool = ctx.data::<PgPool>()?;
        let audit = services::create_audit(pool, &input).await?;
        Ok(audit)
    }
}

pub type AppSchema = async_graphql::Schema<QueryRoot, MutationRoot, async_graphql::EmptySubscription>;
