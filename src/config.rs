use anyhow::{anyhow, Result};
use globset::Glob;
use serde_derive::Deserialize;
use std::path::PathBuf;
use tracing::trace;

#[derive(Debug, Deserialize)]
pub struct Root<'bucket> {
    #[serde(borrow, flatten)]
    pub bucket: Bucket<'bucket>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ConfigurablePath {
    pub path: PathBuf,
    pub recursive: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum PathType {
    Plain(PathBuf),
    Configurable(ConfigurablePath),
}

#[derive(Debug, Deserialize)]
pub struct Bucket<'bucket> {
    pub name: Option<&'bucket str>,
    pub sources: Vec<PathType>,
    pub target: PathBuf,
    pub file_filter: Option<&'bucket str>,
}

impl Root<'_> {
    pub fn is_match(&self, file_name: &str) -> bool {
        if let Some(pattern) = self.bucket.file_filter {
            if !pattern.is_empty() {
                if let Ok(glob) = Glob::new(pattern) {
                    return glob.compile_matcher().is_match(file_name);
                }
            }
        };
        true
    }
}

pub fn get_path() -> Result<PathBuf> {
    let config_path = if let Ok(path) = std::env::var("RUCKSACK_CONFIG") {
        PathBuf::from(path)
    } else {
        let mut path = dirs::config_dir().ok_or_else(|| anyhow!("Failed to get config dir"))?;
        path.push("rucksack");
        path.set_extension("toml");
        path
    };
    trace!("Config file: {}", config_path.to_string_lossy());
    Ok(config_path)
}

#[cfg(test)]
mod test {
    use super::{ConfigurablePath, PathType, Root};
    use assay::assay;
    use basic_toml::from_str;
    use std::path::PathBuf;

    #[assay]
    fn parse() {
        let config = r#"
        name = "Screenshots"
        sources = [
            "/mnt/data/Game 1/screenshots",
            { path = "/mnt/data/Game 2/screenshots", recursive = true }
        ]
        target = "/home/test/screenshots"
        file_filter = "*.mp4"
        "#;
        let config: Root<'_> = from_str(config)?;
        let bucket = &config.bucket;
        assert_eq!(Some("Screenshots"), bucket.name);
        assert_eq!(
            vec![
                PathType::Plain(PathBuf::from("/mnt/data/Game 1/screenshots")),
                PathType::Configurable(ConfigurablePath {
                    path: PathBuf::from("/mnt/data/Game 2/screenshots"),
                    recursive: true,
                }),
            ],
            bucket.sources
        );
        assert_eq!(PathBuf::from("/home/test/screenshots"), bucket.target);
        assert_eq!(Some("*.mp4"), config.bucket.file_filter);
        assert!(config.is_match("1.mp4"));
        assert!(!config.is_match("1.png"));
    }
}
