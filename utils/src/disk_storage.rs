//! Utilities for storing a struct in a file on the disk.
//! The struct should implement Serialize and Deserialize from serde.
//! Supported file formats are TOML and YAML.

use std::{fmt::Debug, fs, path::PathBuf};

use directories::BaseDirs;
use serde::{de::DeserializeOwned, Serialize};

pub enum FileFormat {
    TOML,
    YAML,
}

pub trait DiskStorageInterface
where
    Self: Sized + Debug + Default + Serialize + DeserializeOwned,
{
    const FILE_NAME: &'static str;
    const FORMAT: FileFormat;

    /// Get the path to the file
    fn path() -> crate::Result<PathBuf> {
        let dirs = BaseDirs::new().ok_or(crate::Error::BaseDirsFailed)?;
        let path = dirs
            .home_dir()
            .join(".gm")
            .join(Self::FILE_NAME)
            .with_extension(match Self::FORMAT {
                FileFormat::TOML => "toml".to_string(),
                FileFormat::YAML => "yaml".to_string(),
            });
        Ok(path)
    }

    /// Load the content from the file if it exists otherwise return the default value
    fn load() -> crate::Result<Self> {
        let path = Self::path()?;

        if path.exists() {
            let content = fs::read_to_string(&path)
                .map_err(|e| crate::Error::FileReadFailed(path.clone(), e))?;

            match Self::FORMAT {
                FileFormat::TOML => {
                    toml::from_str(&content).map_err(|e| crate::Error::TomlParsingFailed(path, e))
                }
                FileFormat::YAML => serde_yaml::from_str(&content)
                    .map_err(|e| crate::Error::YamlParsingFailed(path, e)),
            }
        } else {
            Ok(Self::default())
        }
    }

    /// Save content to a file, creating the directories and file as necessary
    fn save(&self) -> crate::Result<()> {
        let path = Self::path()?;

        if let Some(parent) = path.parent() {
            // Ensure config directory exists
            fs::create_dir_all(parent)
                .map_err(|e| crate::Error::CreateDirAllFailed(path.clone(), e))?;
        }

        let content = match Self::FORMAT {
            FileFormat::TOML => toml::to_string_pretty(self)
                .map_err(|e| crate::Error::TomlFormattingFailed(format!("{self:?}"), e))?,
            FileFormat::YAML => serde_yaml::to_string(self)
                .map_err(|e| crate::Error::YamlFormattingFailed(format!("{self:?}"), e))?,
        };

        fs::write(&path, content).map_err(|e| crate::Error::FileWriteFailed(path, e))?;

        Ok(())
    }
}
