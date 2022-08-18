use anyhow::{Context, Result};
use serde::Deserialize;
use tracing::trace;
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
        let resolved = client
            .resolve_root(CanonicalPath::canonicalize(path).context(format!("{}", path.display()))?)
            .await?;
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
