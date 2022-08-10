use anyhow::{anyhow, Result};
use serde_derive::Deserialize;
use std::path::PathBuf;
use tracing::trace;

#[derive(Debug, Deserialize)]
pub struct RootConfig<'bucket> {
    #[serde(borrow)]
    pub bucket: Bucket<'bucket>,
}

#[derive(Debug, Deserialize)]
pub struct Bucket<'bucket> {
    pub name: Option<&'bucket str>,
    pub sources: Vec<PathBuf>,
    pub target: PathBuf,
}

pub fn get_path() -> Result<PathBuf> {
    let mut config_path = dirs::config_dir().ok_or_else(|| anyhow!("Failed to get config dir"))?;
    config_path.push("collector");
    config_path.push("config");
    config_path.set_extension("toml");
    trace!("Config file: {}", config_path.to_string_lossy());
    Ok(config_path)
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use assay::assay;
    use toml::from_str;

    use super::RootConfig;

    #[assay]
    fn parse() {
        let config = r#"
        [bucket]
        name = "Screenshots"
        sources = [
            "/mnt/data/Game 1/screenshots"
        ]
        target = "/home/test/screenshots"
        "#;
        let config: RootConfig<'_> = from_str(config)?;
        let bucket = &config.bucket;
        assert_eq!(Some("Screenshots"), bucket.name);
        assert_eq!(
            vec![PathBuf::from("/mnt/data/Game 1/screenshots")],
            bucket.sources
        );
        assert_eq!(PathBuf::from("/home/test/screenshots"), bucket.target);
    }
}
