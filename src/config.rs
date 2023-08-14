use anyhow::{anyhow, Result};
use globset::Glob;
use serde_derive::Deserialize;
use std::path::PathBuf;
use tracing::trace;

#[derive(Debug, Deserialize)]
pub struct Root<'bucket> {
    #[serde(borrow)]
    pub bucket: Bucket<'bucket>,
}

#[derive(Debug, Deserialize)]
pub struct Bucket<'bucket> {
    pub name: Option<&'bucket str>,
    pub sources: Vec<PathBuf>,
    pub target: PathBuf,
    pub file_filter: Option<&'bucket str>,
}

impl<'a> Root<'a> {
    pub fn is_match(&self, file_name: &str) -> bool {
        if let Some(pattern) = self.bucket.file_filter {
            if let Ok(glob) = Glob::new(pattern) {
                return glob.compile_matcher().is_match(file_name);
            }
        };
        true
    }
}

pub fn get_path() -> Result<PathBuf> {
    let config_path = if let Ok(path) = std::env::var("FILE_COLLECTOR_CONFIG") {
        PathBuf::from(path)
    } else {
        let mut path = dirs::config_dir().ok_or_else(|| anyhow!("Failed to get config dir"))?;
        path.push("collector");
        path.push("config");
        path.set_extension("toml");
        path
    };
    trace!("Config file: {}", config_path.to_string_lossy());
    Ok(config_path)
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use assay::assay;
    use basic_toml::from_str;

    use super::Root;

    #[assay]
    fn parse() {
        let config = r#"
        [bucket]
        name = "Screenshots"
        sources = [
            "/mnt/data/Game 1/screenshots"
        ]
        target = "/home/test/screenshots"
        file_filter = "*.mp4"
        "#;
        let config: Root<'_> = from_str(config)?;
        let bucket = &config.bucket;
        assert_eq!(Some("Screenshots"), bucket.name);
        assert_eq!(
            vec![PathBuf::from("/mnt/data/Game 1/screenshots")],
            bucket.sources
        );
        assert_eq!(PathBuf::from("/home/test/screenshots"), bucket.target);
        assert_eq!(Some("*.mp4"), config.bucket.file_filter);
        assert!(config.is_match("1.mp4"));
        assert!(!config.is_match("1.png"));
    }
}
