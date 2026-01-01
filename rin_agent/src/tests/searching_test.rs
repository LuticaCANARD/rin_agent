#[cfg(test)]
use crate::api::google_searching::google_searching;
use crate::libs::logger::{LOGGER, LogLevel};
#[tokio::test]
async fn searching_test() {
    dotenv::dotenv().ok(); 
    LOGGER.log(LogLevel::Debug, "searching_test start");
    let searching_result = google_searching("rust programming language".to_string()).await;
    match searching_result {
        Ok(result) => {
            LOGGER.log(LogLevel::Debug, format!("searching_test result: {:?}", result).as_str());
            for item in result {
                LOGGER.log(LogLevel::Debug, format!("searching_test item: {:?}\n", item).as_str());
            }

        }
        Err(e) => {
            LOGGER.log(LogLevel::Error, format!("searching_test error: {}", e).as_str());
            panic!("searching_test failed: {}", e);
        }
    }
}
#[tokio::test]
async fn get_document(){
    
}