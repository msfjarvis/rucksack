mod config;
mod logging;
mod watch;

use crate::config::{get_path, PathType, Root};
use crate::watch::generate_subscriptions;
use anyhow::{anyhow, Context, Result};
use futures::future::select_all;
use std::fs::File;
use std::path::Path;
use tracing::{debug, trace};
use watchman_client::{prelude::*, SubscriptionData};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    logging::init()?;
    run().await
}

fn relative_depth(parent: &Path, child: &Path) -> usize {
    child
        .strip_prefix(parent)
        .ok()
        .map_or(1, |relative_path| relative_path.iter().count())
}

async fn run() -> Result<()> {
    let config_path = get_path()?;
    let config_str = std::fs::read_to_string(config_path.as_path()).unwrap_or_default();
    let config: Root<'_> = basic_toml::from_str(&config_str)?;

    if !config.bucket.target.exists() {
        return Err(anyhow!("{} does not exist", config.bucket.target.display()));
    }

    let client = Connector::new().connect().await?;
    let mut subs = generate_subscriptions(&client, &config.bucket).await?;

    loop {
        let subscription_futures = subs
            .iter_mut()
            .map(|sub| Box::pin(sub.next()))
            .collect::<Vec<_>>();
        let (resolved, index, _) = select_all(subscription_futures).await;
        if let Ok(SubscriptionData::FilesChanged(event)) = resolved {
            if let Some(files) = event.files {
                for file in &files {
                    let name = file.name.as_path();
                    let exists = *file.exists;
                    let empty = *file.size == 0;
                    if exists && !empty && config.is_match(name.to_str().unwrap()) {
                        let (raw_source, recursive) = match config.bucket.sources[index].clone() {
                            PathType::Plain(path_buf) => (path_buf, true),
                            PathType::Configurable(configurable_path) => {
                                (configurable_path.path, configurable_path.recursive)
                            }
                        };
                        let source = raw_source.join(name);
                        // We skip directories, and non-immediate children of the source
                        // if the source requests lookups to not be recursive.
                        if source.is_dir()
                            || (relative_depth(&raw_source, &source) > 1 && !recursive)
                        {
                            continue;
                        }
                        let source = source.as_path();
                        let target = config.bucket.target.join(source.file_name().unwrap());
                        let target = target.as_path();

                        let mut src_file = File::open(source)
                            .context(format!("failed to open {}", source.display()))?;
                        let mut dst_file = File::create(target)
                            .context(format!("failed to open {}", target.display()))?;

                        debug!("Moving {} to {}", source.display(), target.display());
                        std::io::copy(&mut src_file, &mut dst_file).context(format!(
                            "src={}, dest={}",
                            source.display(),
                            target.display(),
                        ))?;
                        src_file
                            .sync_all()
                            .context(format!("failed to fsync {}", source.display()))?;
                        dst_file
                            .sync_all()
                            .context(format!("failed to fsync {}", target.display()))?;

                        let src_len = src_file
                            .metadata()
                            .context(format!("{} does not exist", source.display()))?
                            .len();
                        let dst_len = dst_file
                            .metadata()
                            .context(format!("{} does not exist", target.display()))?
                            .len();

                        if src_len != dst_len {
                            return Err(anyhow!("Destination file length does not match! Source file was {src_len} bytes but {dst_len} bytes were written"));
                        }

                        std::fs::remove_file(source)
                            .context(format!("failed to remove {}", source.display()))?;
                        debug!(
                            "Successfully moved {} to {}",
                            name.display(),
                            target.display(),
                        );
                    }
                }
            }
        } else {
            trace!(?resolved);
        }
    }
}
