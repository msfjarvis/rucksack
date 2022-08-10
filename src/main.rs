mod config;
mod watch;

use crate::config::{get_path, Root};
use crate::watch::generate_subscriptions;
use anyhow::{Context, Result};
use futures::future::select_all;
use tracing::metadata::LevelFilter;
use tracing::{debug, error, trace};
use watchman_client::{prelude::*, SubscriptionData};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .pretty()
        .with_max_level(LevelFilter::DEBUG)
        .init();
    if let Err(err) = run().await {
        error!(?err);
        std::process::exit(1);
    }
    Ok(())
}

async fn run() -> Result<()> {
    let config_path = get_path()?;
    let config_str = std::fs::read_to_string(config_path.as_path()).unwrap_or_default();
    let config: Root<'_> = toml::from_str(&config_str)?;

    let client = Connector::new().connect().await?;
    let mut subs = generate_subscriptions(&client, &config.bucket).await?;

    loop {
        let subscription_futures = subs
            .iter_mut()
            .map(|sub| Box::pin(sub.next()))
            .collect::<Vec<_>>();
        let (resolved, index, _) = select_all(subscription_futures).await;
        let resolved = resolved?;
        if let SubscriptionData::FilesChanged(event) = resolved {
            if let Some(files) = event.files {
                for file in &files {
                    debug!(?file);
                    if *file.exists && *file.size > 0 {
                        let mut source = config.bucket.sources[index].clone();
                        let mut target = config.bucket.target.clone();
                        source.push(file.name.as_os_str());
                        target.push(file.name.as_os_str());
                        std::fs::copy(source.clone(), target.clone()).context(format!(
                            "src={}, dest={}",
                            source.clone().display(),
                            target.clone().display()
                        ))?;
                        std::fs::remove_file(source.clone())
                            .context(format!("{}", source.clone().display()))?;
                    }
                }
            }
        } else {
            trace!(?resolved);
        };
    }
}
