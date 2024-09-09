use std::env;
use std::error::Error;

/// A struct to hold retention configuration parameters.
#[derive(Clone)]
pub struct RetentionConfig {
    pub retention_day: usize,
    pub retention_week: usize,
    pub retention_month: usize,
    pub retention_year: usize,
}

impl RetentionConfig {
    /// Creates a new `RetentionConfig` instance by loading values from environment variables.
    ///
    /// This method reads the following environment variables:
    ///
    /// - `RETENTION_DAY`: The number of backups to retain for the current day.
    /// - `RETENTION_WEEK`: The number of backups to retain for the current week, excluding today.
    /// - `RETENTION_MONTH`: The number of backups to retain for the current month, excluding the current week.
    /// - `RETENTION_YEAR`: The number of backups to retain for the current year, excluding the current month.
    ///
    /// If an environment variable is not set, it will use a default value of `usize::MAX` (infinity).
    ///
    /// # Errors
    ///
    /// Returns an `Err` if any of the environment variables cannot be parsed as `usize`.
    pub fn new_from_env() -> Result<Self, Box<dyn Error>> {
        let retention_day = env::var("RETENTION_DAY").ok()
            .and_then(|val| val.parse::<usize>().ok())
            .unwrap_or(usize::MAX);

        let retention_week = env::var("RETENTION_WEEK").ok()
            .and_then(|val| val.parse::<usize>().ok())
            .unwrap_or(usize::MAX);

        let retention_month = env::var("RETENTION_MONTH").ok()
            .and_then(|val| val.parse::<usize>().ok())
            .unwrap_or(usize::MAX);

        let retention_year = env::var("RETENTION_YEAR").ok()
            .and_then(|val| val.parse::<usize>().ok())
            .unwrap_or(usize::MAX);

        Ok(Self { retention_day, retention_week, retention_month, retention_year })
    }
}
