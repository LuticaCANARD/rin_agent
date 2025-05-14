use redis::{Client, Commands};
use std::{env, sync::{Arc, Mutex}};



pub struct RedisDriver {
    client: redis::Client,
    pool : r2d2::Pool<Client>,
}
impl RedisDriver {
    pub fn new(redis_url:String) -> Self {
        let client = redis::Client::open(redis_url.clone()).expect("Invalid Redis URL");
        let pool = r2d2::Pool::builder()
            .build(client.clone())
            .unwrap();
        RedisDriver {
            client,
            pool,
        }
    }
    pub fn get_pool(&self) -> r2d2::Pool<Client> {
        self.pool.clone()
    }
    pub fn get_client(&self) -> redis::Client {
        self.client.clone()
    }
    pub fn get_connection(&self) -> redis::Connection {
        self.client.get_connection().expect("Failed to connect to Redis")
    }
}

