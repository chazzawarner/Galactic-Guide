//! Configuration read from environment variables.

/// Worker runtime configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// PostgreSQL connection string (required).
    pub database_url: String,

    /// Redis connection URL (required).
    pub redis_url: String,

    /// Unique name for this worker instance within the consumer group.
    ///
    /// Defaults to the hostname if `WORKER_NAME` is not set.
    pub worker_name: String,
}

impl Config {
    /// Build a [`Config`] from environment variables.
    ///
    /// # Errors
    /// Returns an error if `DATABASE_URL` or `REDIS_URL` are missing.
    pub fn from_env() -> anyhow::Result<Self> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL environment variable is required"))?;
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379/0".to_owned());
        let worker_name = std::env::var("WORKER_NAME").unwrap_or_else(|_| {
            // Use the OS hostname via gethostname syscall as a default worker name.
            // SAFETY: gethostname is a well-known POSIX function; the buffer is
            // sized to HOST_NAME_MAX (255) + 1 null terminator.
            // gethostname returns 0 on success, -1 on error.
            #[cfg(unix)]
            {
                let mut buf = [0u8; 256];
                let rc = unsafe {
                    libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len())
                };
                if rc == 0 {
                    // Find the NUL terminator and convert to a Rust String.
                    let nul = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
                    String::from_utf8_lossy(&buf[..nul]).into_owned()
                } else {
                    "worker-1".to_owned()
                }
            }
            #[cfg(not(unix))]
            {
                "worker-1".to_owned()
            }
        });
        Ok(Self {
            database_url,
            redis_url,
            worker_name,
        })
    }
}
