use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rusqlite::{params, Connection};
use serde::Serialize;
use serde_json::Value;
use tracing::{debug, warn};

const DB_FILE_NAME: &str = "llm_stats.sqlite";

#[cfg(DEVELOPMENT)]
const DEFAULT_STATS_DIR: &str = ".vibeterminal-dev";
#[cfg(not(DEVELOPMENT))]
const DEFAULT_STATS_DIR: &str = ".vibeterminal";

#[derive(Debug, Clone)]
pub struct LlmRequestContext {
    pub category: String,
}

#[derive(Debug, Clone)]
pub struct LlmRequestStats {
    pub addon_name: String,
    pub model_name: String,
    pub category: String,
    pub requested_at_ms: i64,
    pub duration_ms: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub tokens_estimated: bool,
    pub success: bool,
    pub error: Option<String>,
}

pub fn record_llm_request(stats: &LlmRequestStats) {
    if let Err(error) = record_llm_request_inner(stats) {
        warn!("Failed to record LLM request stats: {}", error);
    }
}

pub fn unix_timestamp_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(i64::MAX as u128) as i64)
        .unwrap_or(0)
}

pub fn duration_millis(duration: Duration) -> i64 {
    duration.as_millis().min(i64::MAX as u128) as i64
}

pub fn estimate_tokens(text: &str) -> i64 {
    let chars = text.chars().count() as i64;
    chars.saturating_add(3) / 4
}

pub fn extract_usage_tokens(value: &Value) -> Option<(i64, i64, i64)> {
    let usage = value.get("usage")?;
    let input = first_i64(
        usage,
        &[
            "prompt_tokens",
            "input_tokens",
            "total_input_tokens",
            "cache_creation_input_tokens",
        ],
    )?;
    let output = first_i64(
        usage,
        &[
            "completion_tokens",
            "output_tokens",
            "total_output_tokens",
            "cache_read_input_tokens",
        ],
    )
    .unwrap_or(0);
    let total = first_i64(usage, &["total_tokens"]).unwrap_or_else(|| input.saturating_add(output));
    Some((input, output, total))
}

fn first_i64(value: &Value, keys: &[&str]) -> Option<i64> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(Value::as_i64))
}

fn record_llm_request_inner(stats: &LlmRequestStats) -> rusqlite::Result<()> {
    let Some(db_path) = stats_db_path() else {
        debug!("Skipping LLM request stats: USERPROFILE/HOME is not set");
        return Ok(());
    };
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).map_err(sqlite_io_error)?;
    }

    let connection = Connection::open(&db_path)?;
    connection.busy_timeout(Duration::from_secs(2))?;
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS llm_requests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            addon_name TEXT NOT NULL,
            model_name TEXT NOT NULL,
            category TEXT NOT NULL,
            requested_at_ms INTEGER NOT NULL,
            duration_ms INTEGER NOT NULL,
            input_tokens INTEGER NOT NULL,
            output_tokens INTEGER NOT NULL,
            total_tokens INTEGER NOT NULL,
            tokens_estimated INTEGER NOT NULL,
            success INTEGER NOT NULL,
            error TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_llm_requests_requested_at
            ON llm_requests(requested_at_ms DESC);
        CREATE INDEX IF NOT EXISTS idx_llm_requests_category
            ON llm_requests(category, requested_at_ms DESC);",
    )?;
    connection.execute(
        "INSERT INTO llm_requests (
            addon_name,
            model_name,
            category,
            requested_at_ms,
            duration_ms,
            input_tokens,
            output_tokens,
            total_tokens,
            tokens_estimated,
            success,
            error
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            stats.addon_name.as_str(),
            stats.model_name.as_str(),
            stats.category.as_str(),
            stats.requested_at_ms,
            stats.duration_ms,
            stats.input_tokens,
            stats.output_tokens,
            stats.total_tokens,
            stats.tokens_estimated as i64,
            stats.success as i64,
            stats.error.as_deref(),
        ],
    )?;
    Ok(())
}

fn sqlite_io_error(error: std::io::Error) -> rusqlite::Error {
    rusqlite::Error::ToSqlConversionFailure(Box::new(error))
}

fn stats_db_path() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty())
        .map(|home| home.join(DEFAULT_STATS_DIR).join(DB_FILE_NAME))
}

// --- Query types for the stats API ---

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmStatsSummary {
    pub total_requests: i64,
    pub success_count: i64,
    pub error_count: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_tokens: i64,
    pub avg_duration_ms: f64,
    pub estimated_count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmStatsByAddon {
    pub addon_name: String,
    pub request_count: i64,
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub success_count: i64,
    pub avg_duration_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmStatsByCategory {
    pub category: String,
    pub request_count: i64,
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub success_count: i64,
    pub avg_duration_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmStatsByModel {
    pub model_name: String,
    pub request_count: i64,
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub success_count: i64,
    pub avg_duration_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmStatsTimeBucket {
    pub bucket: String,
    pub request_count: i64,
    pub total_tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub success_count: i64,
    pub avg_duration_ms: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmStatsResponse {
    pub summary: LlmStatsSummary,
    pub by_addon: Vec<LlmStatsByAddon>,
    pub by_category: Vec<LlmStatsByCategory>,
    pub by_model: Vec<LlmStatsByModel>,
    pub time_series: Vec<LlmStatsTimeBucket>,
}

pub fn query_llm_stats(
    from_ms: Option<i64>,
    to_ms: Option<i64>,
    category: Option<&str>,
    addon_name: Option<&str>,
) -> Result<LlmStatsResponse, String> {
    let db_path = stats_db_path().ok_or("stats database path not available")?;
    if !db_path.exists() {
        return Ok(empty_stats_response());
    }

    let conn = Connection::open(&db_path).map_err(|e| e.to_string())?;
    conn.busy_timeout(Duration::from_secs(2))
        .map_err(|e| e.to_string())?;

    let where_clause = build_where(from_ms, to_ms, category, addon_name);

    let summary = query_summary(&conn, &where_clause)?;
    let by_addon = query_by_addon(&conn, &where_clause)?;
    let by_category = query_by_category(&conn, &where_clause)?;
    let by_model = query_by_model(&conn, &where_clause)?;
    let time_series = query_time_series(&conn, &where_clause, from_ms, to_ms)?;

    Ok(LlmStatsResponse {
        summary,
        by_addon,
        by_category,
        by_model,
        time_series,
    })
}

fn empty_stats_response() -> LlmStatsResponse {
    LlmStatsResponse {
        summary: LlmStatsSummary {
            total_requests: 0,
            success_count: 0,
            error_count: 0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_tokens: 0,
            avg_duration_ms: 0.0,
            estimated_count: 0,
        },
        by_addon: vec![],
        by_category: vec![],
        by_model: vec![],
        time_series: vec![],
    }
}

fn build_where(
    from_ms: Option<i64>,
    to_ms: Option<i64>,
    category: Option<&str>,
    addon_name: Option<&str>,
) -> String {
    let mut conditions: Vec<String> = vec!["1=1".to_string()];
    if let Some(from) = from_ms {
        conditions.push(format!("requested_at_ms >= {}", from));
    }
    if let Some(to) = to_ms {
        conditions.push(format!("requested_at_ms <= {}", to));
    }
    if let Some(cat) = category {
        conditions.push(format!("category = '{}'", cat.replace('\'', "''")));
    }
    if let Some(addon) = addon_name {
        conditions.push(format!("addon_name = '{}'", addon.replace('\'', "''")));
    }
    conditions.join(" AND ")
}

fn query_summary(conn: &Connection, where_clause: &str) -> Result<LlmStatsSummary, String> {
    let sql = format!(
        "SELECT \
            COUNT(*) as total_requests, \
            COALESCE(SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END), 0) as success_count, \
            COALESCE(SUM(CASE WHEN success = 0 THEN 1 ELSE 0 END), 0) as error_count, \
            COALESCE(SUM(input_tokens), 0) as total_input_tokens, \
            COALESCE(SUM(output_tokens), 0) as total_output_tokens, \
            COALESCE(SUM(total_tokens), 0) as total_tokens, \
            COALESCE(AVG(duration_ms), 0) as avg_duration_ms, \
            COALESCE(SUM(CASE WHEN tokens_estimated = 1 THEN 1 ELSE 0 END), 0) as estimated_count \
        FROM llm_requests WHERE {}",
        where_clause
    );
    conn.query_row(&sql, [], |row| {
        Ok(LlmStatsSummary {
            total_requests: row.get(0)?,
            success_count: row.get(1)?,
            error_count: row.get(2)?,
            total_input_tokens: row.get(3)?,
            total_output_tokens: row.get(4)?,
            total_tokens: row.get(5)?,
            avg_duration_ms: row.get(6)?,
            estimated_count: row.get(7)?,
        })
    })
    .map_err(|e| e.to_string())
}

fn query_by_addon(conn: &Connection, where_clause: &str) -> Result<Vec<LlmStatsByAddon>, String> {
    let sql = format!(
        "SELECT addon_name, COUNT(*), COALESCE(SUM(total_tokens), 0), \
         COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0), \
         COALESCE(SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END), 0), COALESCE(AVG(duration_ms), 0) \
         FROM llm_requests WHERE {} GROUP BY addon_name ORDER BY SUM(total_tokens) DESC",
        where_clause
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(LlmStatsByAddon {
                addon_name: row.get(0)?,
                request_count: row.get(1)?,
                total_tokens: row.get(2)?,
                input_tokens: row.get(3)?,
                output_tokens: row.get(4)?,
                success_count: row.get(5)?,
                avg_duration_ms: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

fn query_by_category(
    conn: &Connection,
    where_clause: &str,
) -> Result<Vec<LlmStatsByCategory>, String> {
    let sql = format!(
        "SELECT category, COUNT(*), COALESCE(SUM(total_tokens), 0), \
         COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0), \
         COALESCE(SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END), 0), COALESCE(AVG(duration_ms), 0) \
         FROM llm_requests WHERE {} GROUP BY category ORDER BY SUM(total_tokens) DESC",
        where_clause
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(LlmStatsByCategory {
                category: row.get(0)?,
                request_count: row.get(1)?,
                total_tokens: row.get(2)?,
                input_tokens: row.get(3)?,
                output_tokens: row.get(4)?,
                success_count: row.get(5)?,
                avg_duration_ms: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

fn query_by_model(conn: &Connection, where_clause: &str) -> Result<Vec<LlmStatsByModel>, String> {
    let sql = format!(
        "SELECT model_name, COUNT(*), COALESCE(SUM(total_tokens), 0), \
         COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0), \
         COALESCE(SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END), 0), COALESCE(AVG(duration_ms), 0) \
         FROM llm_requests WHERE {} GROUP BY model_name ORDER BY SUM(total_tokens) DESC",
        where_clause
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(LlmStatsByModel {
                model_name: row.get(0)?,
                request_count: row.get(1)?,
                total_tokens: row.get(2)?,
                input_tokens: row.get(3)?,
                output_tokens: row.get(4)?,
                success_count: row.get(5)?,
                avg_duration_ms: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

fn query_time_series(
    conn: &Connection,
    where_clause: &str,
    from_ms: Option<i64>,
    to_ms: Option<i64>,
) -> Result<Vec<LlmStatsTimeBucket>, String> {
    // Determine bucket size based on date range
    let range_ms = match (from_ms, to_ms) {
        (Some(f), Some(t)) => t - f,
        _ => {
            // Default: last 30 days
            let now = unix_timestamp_millis();
            now - 30 * 24 * 3600 * 1000
        }
    };
    let day_ms: i64 = 24 * 3600 * 1000;
    let bucket_fmt = if range_ms > 90 * day_ms {
        "%Y-%m" // monthly
    } else if range_ms > 3 * day_ms {
        "%Y-%m-%d" // daily
    } else {
        "%Y-%m-%d %H:00" // hourly
    };

    let sql = format!(
        "SELECT strftime('{}', requested_at_ms / 1000, 'unixepoch') as bucket, \
         COUNT(*), COALESCE(SUM(total_tokens), 0), \
         COALESCE(SUM(input_tokens), 0), COALESCE(SUM(output_tokens), 0), \
         COALESCE(SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END), 0), COALESCE(AVG(duration_ms), 0) \
         FROM llm_requests WHERE {} GROUP BY bucket ORDER BY bucket",
        bucket_fmt, where_clause
    );
    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(LlmStatsTimeBucket {
                bucket: row.get(0)?,
                request_count: row.get(1)?,
                total_tokens: row.get(2)?,
                input_tokens: row.get(3)?,
                output_tokens: row.get(4)?,
                success_count: row.get(5)?,
                avg_duration_ms: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_over_empty_table_returns_zeros() {
        // Regression: an empty result set makes `SUM(CASE WHEN ...)` return
        // SQL NULL, which previously failed query_summary with
        // "Invalid column type Null at index: 1, name: success_count".
        // COALESCE(..., 0) keeps every aggregate a non-null integer.
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE llm_requests (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                addon_name TEXT NOT NULL,
                model_name TEXT NOT NULL,
                category TEXT NOT NULL,
                requested_at_ms INTEGER NOT NULL,
                duration_ms INTEGER NOT NULL,
                input_tokens INTEGER NOT NULL,
                output_tokens INTEGER NOT NULL,
                total_tokens INTEGER NOT NULL,
                tokens_estimated INTEGER NOT NULL,
                success INTEGER NOT NULL,
                error TEXT
            );",
        )
        .unwrap();

        let summary = query_summary(&conn, "1=1").unwrap();
        assert_eq!(summary.total_requests, 0);
        assert_eq!(summary.success_count, 0);
        assert_eq!(summary.error_count, 0);
        assert_eq!(summary.estimated_count, 0);
    }

    #[test]
    fn extracts_openai_usage_tokens() {
        let value = serde_json::json!({
            "usage": {
                "prompt_tokens": 12,
                "completion_tokens": 5,
                "total_tokens": 17
            }
        });

        assert_eq!(extract_usage_tokens(&value), Some((12, 5, 17)));
    }

    #[test]
    fn estimates_tokens_from_text_length() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("abcd"), 1);
        assert_eq!(estimate_tokens("abcde"), 2);
    }
}
