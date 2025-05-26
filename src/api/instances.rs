
use std::sync::LazyLock;

use rs_ervice::{RSContext, RSContextBuilder};
use tokio::{runtime::Runtime, sync::OnceCell};

use super::schedule::ScheduleService;

pub static RIN_SERVICES: OnceCell<RSContext> = OnceCell::const_new();

pub async fn init_rin_services() {
    let ctx = RSContextBuilder::new()
        .register::<ScheduleService>()
        .await
        .build()
        .await
        .expect("Failed to build RIN services context");
    RIN_SERVICES.set(ctx).unwrap_or_else(|_| panic!("RIN_SERVICES already initialized"));
}

pub async fn get_rin_services() -> &'static RSContext {
    if RIN_SERVICES.get().is_none() {
        init_rin_services().await;
    }
    RIN_SERVICES.get().expect("RIN_SERVICES not initialized")
}