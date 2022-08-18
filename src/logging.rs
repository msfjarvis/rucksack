use tracing::dispatcher::SetGlobalDefaultError;
use tracing::subscriber::set_global_default;

#[cfg(not(feature = "journald"))]
fn configure_tracing() -> Result<(), SetGlobalDefaultError> {
    use tracing_subscriber::{fmt::layer, layer::SubscriberExt, registry};

    let stdout_log = layer().pretty();
    let subscriber = registry().with(stdout_log);
    set_global_default(subscriber)
}

#[cfg(feature = "journald")]
fn configure_tracing() -> Result<(), SetGlobalDefaultError> {
    use tracing_journald::layer;
    use tracing_subscriber::{layer::SubscriberExt, registry};

    let subscriber = registry().with(layer().unwrap());
    set_global_default(subscriber)
}

pub fn init() -> Result<(), SetGlobalDefaultError> {
    configure_tracing()
}
