#[macro_export]
macro_rules! lazy_awaiter {
    ($type:ty, $func:expr) => {
        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(async { $func().await })
    };
}