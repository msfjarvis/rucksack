mod config;

use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;
use watchman_client::{prelude::*, SubscriptionData};

query_result_type! {
    struct NameAndType {
        name: NameField,
        file_type: FileTypeField,
        exists: ExistsField,
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    if let Err(err) = run().await {
        // Print a prettier error than the default
        eprintln!("{}", err);
        std::process::exit(1);
    }
    Ok(())
}

async fn run() -> Result<()> {
    let path = PathBuf::from(".");
    let client = Connector::new().connect().await?;
    let resolved = client
        .resolve_root(CanonicalPath::canonicalize(path)?)
        .await?;

    let (mut sub, _) = client
        .subscribe::<NameAndType>(&resolved, SubscribeRequest::default())
        .await?;

    loop {
        let item = sub.next().await?;
        match item {
            SubscriptionData::FilesChanged(_) => {}
            _ => println!("{:#?}", item),
        };
    }
}
