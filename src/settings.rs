use config::{Config, Environment};
use config::{ConfigError, FileFormat};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Settings {
    pub backend: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            backend: "github".into(),
        }
    }
}

/// Get the araki configuration settings. In order, this merges
///
/// 1. Default settings
/// 2. User-level araki.toml
/// 3. Local araki.toml
/// 4. Environment variables prefixed with 'ARAKI_'
pub fn get_settings_from_config_dir(config_dir: &Path) -> Result<Settings, ConfigError> {
    Config::builder()
        .add_source(Config::try_from(&Settings::default())?)
        .add_source(config::File::from(config_dir.join("config.toml")).required(false))
        .add_source(config::File::new("araki", FileFormat::Toml).required(false))
        .add_source(Environment::with_prefix("ARAKI"))
        .build()?
        .try_deserialize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use tempfile::tempdir;

    #[test]
    fn no_config_is_same_as_default() -> Result<(), Box<dyn Error>> {
        let defaults: Settings = Default::default();
        let tmp = tempdir()?;
        let result = get_settings_from_config_dir(tmp.path())?;
        assert_eq!(result, defaults);
        Ok(())
    }
}
