use std::path::Path;

use thiserror::Error;
use tracing::debug;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to read secret ({1}, {2}): {0}")]
    Io(std::io::Error, String, String),
}

pub fn read(name: impl Into<String>, file: impl Into<String>) -> Result<String, Error> {
    let name: String = name.into();
    let file: String = file.into();

    #[cfg(debug_assertions)]
    {
        let variable_name = format!(
            "SECRET_{}_{}",
            name.replace('-', "_"),
            file.replace('-', "_")
        );
        debug!(
            "Debug build, reading secret from env variable: {}",
            &variable_name
        );
        let env = std::env::var(variable_name);

        if let Ok(value) = env {
            return Ok(value);
        }
    }

    let path = Path::new("/etc/secrets/")
        .join(name.clone())
        .join(file.clone());

    std::fs::read_to_string(path).map_err(|e| Error::Io(e, name, file))
}
