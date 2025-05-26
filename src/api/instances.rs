
use std::sync::LazyLock;

use rs_ervice::{RSContext, RSContextBuilder};
use tokio::{runtime::Runtime, sync::OnceCell};

use super::schedule::ScheduleService;

pub static RIN_SERVICES: OnceCell<RSContext> = OnceCell::const_new();

pub async fn init_rin_services() {
    let ctx = RSContextBuilder::new()
        .register::<ScheduleService>()
        .build()
        .await
        .expect("Failed to build RIN services context");
    RIN_SERVICES.set(ctx).unwrap_or_else(|_| panic!("RIN_SERVICES already initialized"));
}