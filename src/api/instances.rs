
use std::sync::LazyLock;

use rs_ervice::{RSContext, RSContextBuilder};
use tokio::runtime::Runtime;

use super::schedule::ScheduleService;


pub static RIN_SERVICES: LazyLock<RSContext> = LazyLock::new(||
    // 런타임의 block_on 메소드를 사용하여 async 블록을 실행하고 결과를 기다립니다.
    Runtime::new()
    .expect("Failed to create Tokio runtime")
    .block_on(async {
      RSContextBuilder::new()
        .register::<ScheduleService>()
        .build()
        .await
        .expect("Failed to build RIN services context")
    })
);