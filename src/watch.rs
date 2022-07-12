use anyhow::Result;
use serde::Deserialize;
use watchman_client::{prelude::*, Subscription};

use crate::config::Bucket;

query_result_type! {
    pub struct NameAndType {
        pub name: NameField,
        pub file_type: FileTypeField,
        pub exists: ExistsField,
    }
}

pub async fn generate_subscriptions<'a>(
    client: &'a Client,
    bucket: &'a Bucket<'_>,
) -> Result<Vec<Subscription<NameAndType>>> {
    if let Some(name) = bucket.name {
        println!("Generating Watchman subscription for {}", name);
    }
    let mut subs = vec![];
    for path in &bucket.sources {
        let resolved = client
            .resolve_root(CanonicalPath::canonicalize(path)?)
            .await?;
        let (sub, _) = client
            .subscribe::<NameAndType>(&resolved, SubscribeRequest::default())
            .await?;
        subs.push(sub);
    }
    Ok(subs)
}
