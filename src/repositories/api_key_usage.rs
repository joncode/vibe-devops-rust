//! API key usage repository
use anyhow::Result;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{ApiKeyUsage, EndpointStats, HexId, UsageStats};

pub struct ApiKeyUsageRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> ApiKeyUsageRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn record(
        &self, service_account_id: Uuid, endpoint: &str, method: &str,
        status_code: i32, response_time_ms: Option<i32>,
        ip_address: Option<&str>, user_agent: Option<&str>,
    ) -> Result<ApiKeyUsage> {
        let hex_id = ApiKeyUsage::generate_hex_id();
        sqlx::query_as::<_, ApiKeyUsage>(
            r#"INSERT INTO app_api_key_usage (hex_id, service_account_id, endpoint, method, status_code, response_time_ms, ip_address, user_agent)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *"#)
            .bind(&hex_id).bind(service_account_id).bind(endpoint).bind(method)
            .bind(status_code).bind(response_time_ms).bind(ip_address).bind(user_agent)
            .fetch_one(self.pool).await.map_err(Into::into)
    }

    pub async fn get_stats(&self, service_account_id: Uuid) -> Result<UsageStats> {
        let now = Utc::now();
        let hour_ago = now - Duration::hours(1);
        let day_ago = now - Duration::hours(24);

        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM app_api_key_usage WHERE service_account_id = $1 AND deleted_at IS NULL")
            .bind(service_account_id).fetch_one(self.pool).await?;
        let hour: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM app_api_key_usage WHERE service_account_id = $1 AND created_at > $2 AND deleted_at IS NULL")
            .bind(service_account_id).bind(hour_ago).fetch_one(self.pool).await?;
        let day: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM app_api_key_usage WHERE service_account_id = $1 AND created_at > $2 AND deleted_at IS NULL")
            .bind(service_account_id).bind(day_ago).fetch_one(self.pool).await?;
        let avg: (Option<f64>,) = sqlx::query_as("SELECT AVG(response_time_ms)::FLOAT8 FROM app_api_key_usage WHERE service_account_id = $1 AND response_time_ms IS NOT NULL AND deleted_at IS NULL")
            .bind(service_account_id).fetch_one(self.pool).await?;
        let errors: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM app_api_key_usage WHERE service_account_id = $1 AND status_code >= 400 AND deleted_at IS NULL")
            .bind(service_account_id).fetch_one(self.pool).await?;

        let eps: Vec<(String, String, i64, Option<f64>, i64)> = sqlx::query_as(
            r#"SELECT endpoint, method, COUNT(*), AVG(response_time_ms)::FLOAT8, SUM(CASE WHEN status_code >= 400 THEN 1 ELSE 0 END)
            FROM app_api_key_usage WHERE service_account_id = $1 AND deleted_at IS NULL GROUP BY endpoint, method ORDER BY 3 DESC LIMIT 10"#)
            .bind(service_account_id).fetch_all(self.pool).await?;

        Ok(UsageStats {
            total_requests: total.0,
            requests_last_hour: hour.0,
            requests_last_24h: day.0,
            avg_response_time_ms: avg.0,
            error_rate: if total.0 > 0 { errors.0 as f64 / total.0 as f64 } else { 0.0 },
            most_used_endpoints: eps.into_iter().map(|(e, m, c, a, err)| EndpointStats {
                endpoint: e, method: m, request_count: c, avg_response_time_ms: a, error_count: err
            }).collect(),
        })
    }
}
