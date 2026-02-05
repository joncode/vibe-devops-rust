//! API Key Usage repository

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{ApiKeyUsage, ApiKeyUsageStats, EndpointStats, HexId};

/// API Key Usage repository
pub struct ApiKeyUsageRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> ApiKeyUsageRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Record API key usage
    pub async fn record(
        &self,
        service_account_id: Uuid,
        endpoint: &str,
        method: &str,
        status_code: i32,
        response_time_ms: Option<i32>,
        request_metadata: serde_json::Value,
    ) -> Result<ApiKeyUsage> {
        let hex_id = ApiKeyUsage::generate_hex_id();

        let usage = sqlx::query_as::<_, ApiKeyUsage>(
            r#"
            INSERT INTO app_api_key_usage (
                hex_id, service_account_id, endpoint, method, status_code,
                response_time_ms, request_metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(&hex_id)
        .bind(service_account_id)
        .bind(endpoint)
        .bind(method)
        .bind(status_code)
        .bind(response_time_ms)
        .bind(&request_metadata)
        .fetch_one(self.pool)
        .await?;

        Ok(usage)
    }

    /// Count recent calls for rate limiting
    pub async fn count_recent_calls(
        &self,
        service_account_id: Uuid,
        since: DateTime<Utc>,
    ) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM app_api_key_usage
            WHERE service_account_id = $1 
              AND created_at >= $2
              AND deleted_at IS NULL
            "#,
        )
        .bind(service_account_id)
        .bind(since)
        .fetch_one(self.pool)
        .await?;

        Ok(count.0)
    }

    /// Count calls in the last minute for rate limiting
    pub async fn count_calls_last_minute(&self, service_account_id: Uuid) -> Result<i64> {
        let since = Utc::now() - Duration::minutes(1);
        self.count_recent_calls(service_account_id, since).await
    }

    /// Find usage by hex_id
    pub async fn find_by_hex_id(&self, hex_id: &str) -> Result<Option<ApiKeyUsage>> {
        let usage = sqlx::query_as::<_, ApiKeyUsage>(
            r#"
            SELECT * FROM app_api_key_usage
            WHERE hex_id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(hex_id)
        .fetch_optional(self.pool)
        .await?;

        Ok(usage)
    }

    /// List usage by service account with pagination
    pub async fn find_by_service_account(
        &self,
        service_account_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<ApiKeyUsage>> {
        let usages = sqlx::query_as::<_, ApiKeyUsage>(
            r#"
            SELECT * FROM app_api_key_usage
            WHERE service_account_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(service_account_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await?;

        Ok(usages)
    }

    /// Get usage statistics for a service account
    pub async fn get_stats(&self, service_account_id: Uuid) -> Result<ApiKeyUsageStats> {
        // Total calls
        let total_calls: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM app_api_key_usage
            WHERE service_account_id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(service_account_id)
        .fetch_one(self.pool)
        .await?;

        // Calls in last hour
        let hour_ago = Utc::now() - Duration::hours(1);
        let calls_last_hour: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM app_api_key_usage
            WHERE service_account_id = $1 
              AND created_at >= $2
              AND deleted_at IS NULL
            "#,
        )
        .bind(service_account_id)
        .bind(hour_ago)
        .fetch_one(self.pool)
        .await?;

        // Calls in last 24h
        let day_ago = Utc::now() - Duration::hours(24);
        let calls_last_24h: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM app_api_key_usage
            WHERE service_account_id = $1 
              AND created_at >= $2
              AND deleted_at IS NULL
            "#,
        )
        .bind(service_account_id)
        .bind(day_ago)
        .fetch_one(self.pool)
        .await?;

        // Average response time
        let avg_response: (Option<f64>,) = sqlx::query_as(
            r#"
            SELECT AVG(response_time_ms::float) FROM app_api_key_usage
            WHERE service_account_id = $1 
              AND response_time_ms IS NOT NULL
              AND deleted_at IS NULL
            "#,
        )
        .bind(service_account_id)
        .fetch_one(self.pool)
        .await?;

        // Error rate (4xx and 5xx responses)
        let error_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM app_api_key_usage
            WHERE service_account_id = $1 
              AND status_code >= 400
              AND deleted_at IS NULL
            "#,
        )
        .bind(service_account_id)
        .fetch_one(self.pool)
        .await?;

        let error_rate = if total_calls.0 > 0 {
            error_count.0 as f64 / total_calls.0 as f64
        } else {
            0.0
        };

        // Top endpoints
        let top_endpoints = self.get_top_endpoints(service_account_id, 10).await?;

        Ok(ApiKeyUsageStats {
            total_calls: total_calls.0,
            calls_last_hour: calls_last_hour.0,
            calls_last_24h: calls_last_24h.0,
            avg_response_time_ms: avg_response.0,
            error_rate,
            top_endpoints,
        })
    }

    /// Get top endpoints by call count
    async fn get_top_endpoints(
        &self,
        service_account_id: Uuid,
        limit: i64,
    ) -> Result<Vec<EndpointStats>> {
        let rows: Vec<(String, String, i64, Option<f64>, i64)> = sqlx::query_as(
            r#"
            SELECT 
                endpoint,
                method,
                COUNT(*) as call_count,
                AVG(response_time_ms::float) as avg_response_time,
                COUNT(*) FILTER (WHERE status_code >= 400) as error_count
            FROM app_api_key_usage
            WHERE service_account_id = $1 AND deleted_at IS NULL
            GROUP BY endpoint, method
            ORDER BY call_count DESC
            LIMIT $2
            "#,
        )
        .bind(service_account_id)
        .bind(limit)
        .fetch_all(self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(
                |(endpoint, method, call_count, avg_response_time_ms, error_count)| EndpointStats {
                    endpoint,
                    method,
                    call_count,
                    avg_response_time_ms,
                    error_count,
                },
            )
            .collect())
    }

    /// Soft delete old usage records (for cleanup)
    pub async fn cleanup_old_records(&self, older_than: DateTime<Utc>) -> Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE app_api_key_usage
            SET deleted_at = NOW()
            WHERE created_at < $1 AND deleted_at IS NULL
            "#,
        )
        .bind(older_than)
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// List all usage with pagination (for admin)
    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<ApiKeyUsage>> {
        let usages = sqlx::query_as::<_, ApiKeyUsage>(
            r#"
            SELECT * FROM app_api_key_usage
            WHERE deleted_at IS NULL
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await?;

        Ok(usages)
    }
}
