use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::time::Duration;

pub type DbPool = Pool<Postgres>;

/// Environment variable bounding how long a request waits for a pooled
/// database connection, including opening a new one, before it fails.
pub const DATABASE_ACQUIRE_TIMEOUT_ENV: &str = "DATABASE_ACQUIRE_TIMEOUT_MS";

/// Default for [`DATABASE_ACQUIRE_TIMEOUT_ENV`]: long enough for a healthy
/// pooler under load, short enough that an outage fails requests quickly
/// instead of holding handlers for sqlx's 30 s default.
pub const DEFAULT_DATABASE_ACQUIRE_TIMEOUT: Duration = Duration::from_millis(3_000);

/// Parse a [`DATABASE_ACQUIRE_TIMEOUT_ENV`] value. Missing, empty, zero or
/// unparsable values fall back to [`DEFAULT_DATABASE_ACQUIRE_TIMEOUT`]: a
/// zero timeout would fail every acquire, so it is never honoured.
pub fn parse_database_acquire_timeout(value: Option<&str>) -> Duration {
    match value.map(str::trim).map(str::parse::<u64>) {
        Some(Ok(ms)) if ms > 0 => Duration::from_millis(ms),
        Some(_) => {
            tracing::warn!(
                "{DATABASE_ACQUIRE_TIMEOUT_ENV} must be a positive integer of milliseconds; using {} ms",
                DEFAULT_DATABASE_ACQUIRE_TIMEOUT.as_millis()
            );
            DEFAULT_DATABASE_ACQUIRE_TIMEOUT
        }
        None => DEFAULT_DATABASE_ACQUIRE_TIMEOUT,
    }
}

/// The request-path acquire timeout, read from [`DATABASE_ACQUIRE_TIMEOUT_ENV`].
pub fn database_acquire_timeout() -> Duration {
    parse_database_acquire_timeout(std::env::var(DATABASE_ACQUIRE_TIMEOUT_ENV).ok().as_deref())
}

pub async fn create_pool(connection_string: &str) -> Result<DbPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(50)
        .acquire_timeout(database_acquire_timeout())
        .connect(connection_string)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquire_timeout_defaults_and_overrides() {
        assert_eq!(
            parse_database_acquire_timeout(None),
            DEFAULT_DATABASE_ACQUIRE_TIMEOUT
        );
        assert_eq!(
            parse_database_acquire_timeout(Some("1500")),
            Duration::from_millis(1500)
        );
        assert_eq!(
            parse_database_acquire_timeout(Some(" 250 ")),
            Duration::from_millis(250)
        );
        for bad in ["", "0", "-5", "3s", "abc"] {
            assert_eq!(
                parse_database_acquire_timeout(Some(bad)),
                DEFAULT_DATABASE_ACQUIRE_TIMEOUT,
                "{bad:?}"
            );
        }
    }
}
