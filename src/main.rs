mod config;
mod logging;
mod watch;

use crate::config::{get_path, Root};
use crate::watch::generate_subscriptions;
use anyhow::{Context, Result};
use futures::future::select_all;
use tracing::{debug, trace};
use watchman_client::{prelude::*, SubscriptionData};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    logging::init()?;
    run().await
}

async fn run() -> Result<()> {
    let config_path = get_path()?;
    let config_str = std::fs::read_to_string(config_path.as_path()).unwrap_or_default();
    let config: Root<'_> = basic_toml::from_str(&config_str)?;

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
                        let source = config.bucket.sources[index].join(name);
                        let source = source.as_path();
                        let target = config.bucket.target.join(source.file_name().unwrap());
                        let target = target.as_path();
                        debug!("Moving {} to {}", source.display(), target.display());
                        std::fs::copy(source, target).context(format!(
                            "src={}, dest={}",
                            source.display(),
                            target.display()
                        ))?;
                        std::fs::remove_file(source).context(format!("{}", source.display()))?;
                        debug!(
                            "Successfully moved {} to {}",
                            name.display(),
                            target.display()
                        );
                    }
                }
            }
        } else {
            trace!(?resolved);
        };
    }
}
