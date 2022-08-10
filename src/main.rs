mod config;
mod watch;

use crate::config::{get_path, RootConfig};
use crate::watch::generate_subscriptions;
use anyhow::Result;
use futures::future::select_all;
use tracing::metadata::LevelFilter;
use tracing::{error, info, trace};
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
    let config: RootConfig<'_> = toml::from_str(&config_str)?;

    let client = Connector::new().connect().await?;
    let mut subs = generate_subscriptions(&client, &config.bucket).await?;

    loop {
        let subscription_futures = subs
            .iter_mut()
            .map(|sub| Box::pin(sub.next()))
            .collect::<Vec<_>>();
        let (resolved, index, _) = select_all(subscription_futures).await;
        let resolved = resolved?;
        match resolved {
            SubscriptionData::FilesChanged(event) => {
                if let Some(files) = event.files {
                    for file in files.iter() {
                        let f = &config.bucket.sources[index];
                        info!(?f, ?file);
                    }
                }
            }
            _ => trace!(?resolved),
        };
    }
}
