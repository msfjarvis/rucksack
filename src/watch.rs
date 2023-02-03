use anyhow::{bail, Result};
use serde::Deserialize;
use std::io::ErrorKind;
use tracing::{error, trace};
use watchman_client::{prelude::*, Subscription};

use crate::config::Bucket;

query_result_type! {
    pub struct NameAndType {
        pub name: NameField,
        pub file_type: FileTypeField,
        pub exists: ExistsField,
        pub size: SizeField,
    }
}

pub async fn generate_subscriptions<'a>(
    client: &'a Client,
    bucket: &'a Bucket<'_>,
) -> Result<Vec<Subscription<NameAndType>>> {
    if let Some(name) = bucket.name {
        trace!("Generating Watchman subscriptions for {}", name);
    }
    let mut subs = vec![];
    for path in &bucket.sources {
        let canonical_path = match CanonicalPath::canonicalize(path) {
            Ok(path) => path,
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    error!("Directory {} not found, ignoring...", path.display());
                    continue;
                }
                bail!(err)
            }
        };
        let resolved = client.resolve_root(canonical_path).await?;
        trace!(
            "Adding subscription for {}",
            resolved.path().as_path().display()
        );
        let (sub, _) = client
            .subscribe::<NameAndType>(&resolved, SubscribeRequest::default())
            .await?;
        subs.push(sub);
    }
    Ok(subs)
}
