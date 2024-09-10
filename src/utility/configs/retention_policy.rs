use std::env;
use std::error::Error;

/// A struct to hold retention configuration parameters.
#[derive(Clone)]
pub struct RetentionPolicy {
    pub count: usize,
    pub period: usize,
}

impl RetentionPolicy {
    /// Creates a new `RetentionConfig` instance by loading values from environment variables.
    ///
    /// This method reads the following environment variables:
    ///
    /// - `BACKUP_RETENTION_COUNT`: The maximum number of backups to retain.
    /// - `BACKUP_RETENTION_PERIOD_IN_DAYS`: The number of days to retain backups, deleting backups older than this.
    ///
    /// If an environment variable is not set, it will use a default value.
    ///
    /// # Errors
    ///
    /// Returns an `Err` if any of the environment variables cannot be parsed as `usize`.
    pub fn new_from_env() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            count: Self::parse_env_or_default("BACKUP_RETENTION_COUNT", usize::MAX),
            period: Self::parse_env_or_default("BACKUP_RETENTION_PERIOD_IN_DAYS", usize::MAX),
        })
    }

    /// Creates a `RetentionConfig` instance with no backups ever deleted.
    ///
    /// This represents a configuration where:
    /// - `backup_retention_count` is set to `usize::MAX` (infinity).
    /// - `backup_retention_period` is set to `usize::MAX` (infinity days).
    pub fn new_no_delete() -> Self {
        Self { count: usize::MAX, period: usize::MAX }
    }

    /// Helper function to parse an environment variable as `usize`, defaulting to the provided value if not set or invalid.
    fn parse_env_or_default(var_name: &str, default: usize) -> usize {
        env::var(var_name)
            .ok()
            .and_then(|val| val.parse::<usize>().ok())
            .unwrap_or(default)
    }
}
