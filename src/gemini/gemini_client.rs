pub mod gemini_client{
    use std::io::{stdout, Write};
    use curl::easy::Easy;
    use std::env;
    use std::sync::{Arc, Mutex};

    struct GeminiClient {
        net_client: Easy,
    }

    trait GeminiClientTrait {
        fn new() -> Self;
        fn send_request(&mut self, url: &str) -> Result<String, String>;
        
    }
    impl GeminiClientTrait for GeminiClient {
        fn new() -> Self {
            GeminiClient {
                net_client: Easy::new(),
            }
        }
/**
     * curl "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent?key=${GEMINI_API_KEY}" \
    -H 'Content-Type: application/json' \
    -X POST \
    -d '{
        "contents": [
        {
            "parts": [
            {
                "text": "Write a story about a magic backpack."
            }
            ]
        }
        ]
    }'
 * 
 */
        fn send_request(&mut self, url: &str) -> Result<String, String> {
            let data = Arc::new(Mutex::new(Vec::new()));
            self.net_client.url(url).unwrap();
            let mut transfer = self.net_client.transfer();
            {
                let data = Arc::clone(&data);
                transfer.write_function(move |new_data| {
                    let mut data = data.lock().unwrap();
                    data.extend_from_slice(new_data);
                    Ok(new_data.len())
                }).unwrap();
            }
            transfer.perform().unwrap();
            let response = String::from_utf8(data.lock().unwrap().clone()).unwrap();
            Ok(response)
        }
    } 
}