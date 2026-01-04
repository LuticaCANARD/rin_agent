use anyhow::{Context, Result};
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Redis 연결 관리자
pub struct RedisManager {
    client: Client,
    connection: Arc<Mutex<Option<MultiplexedConnection>>>,
}

impl RedisManager {
    /// 새로운 RedisManager 생성
    /// 
    /// # Arguments
    /// * `redis_url` - Redis 서버 URL (예: "redis://127.0.0.1:6379")
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url)
            .context("Failed to create Redis client")?;
        
        Ok(Self {
            client,
            connection: Arc::new(Mutex::new(None)),
        })
    }

    /// Redis 서버에 연결
    pub async fn connect(&self) -> Result<()> {
        let conn = self.client
            .get_multiplexed_async_connection()
            .await
            .context("Failed to connect to Redis")?;
        
        *self.connection.lock().await = Some(conn);
        Ok(())
    }

    /// 연결 상태 확인
    pub async fn is_connected(&self) -> bool {
        self.connection.lock().await.is_some()
    }

    /// 연결 해제
    pub async fn disconnect(&self) {
        *self.connection.lock().await = None;
    }

    /// Redis PING 명령 실행
    pub async fn ping(&self) -> Result<String> {
        let mut conn_guard = self.connection.lock().await;
        let conn = conn_guard
            .as_mut()
            .context("Not connected to Redis")?;
        
        redis::cmd("PING")
            .query_async(conn)
            .await
            .context("Failed to execute PING")
    }

    /// 모든 키 목록 조회
    pub async fn get_all_keys(&self) -> Result<Vec<String>> {
        let mut conn_guard = self.connection.lock().await;
        let conn = conn_guard
            .as_mut()
            .context("Not connected to Redis")?;
        
        let keys: Vec<String> = conn
            .keys("*")
            .await
            .context("Failed to get keys")?;
        
        Ok(keys)
    }

    /// 특정 패턴의 키 조회
    pub async fn get_keys_by_pattern(&self, pattern: &str) -> Result<Vec<String>> {
        let mut conn_guard = self.connection.lock().await;
        let conn = conn_guard
            .as_mut()
            .context("Not connected to Redis")?;
        
        let keys: Vec<String> = conn
            .keys(pattern)
            .await
            .context("Failed to get keys by pattern")?;
        
        Ok(keys)
    }

    /// 키 값 조회
    pub async fn get_value(&self, key: &str) -> Result<Option<String>> {
        let mut conn_guard = self.connection.lock().await;
        let conn = conn_guard
            .as_mut()
            .context("Not connected to Redis")?;
        
        let value: Option<String> = conn
            .get(key)
            .await
            .context("Failed to get value")?;
        
        Ok(value)
    }

    /// 키 값 설정
    pub async fn set_value(&self, key: &str, value: &str) -> Result<()> {
        let mut conn_guard = self.connection.lock().await;
        let conn = conn_guard
            .as_mut()
            .context("Not connected to Redis")?;
        
        let _: () = conn.set(key, value)
            .await
            .context("Failed to set value")?;
        
        Ok(())
    }
    /// 키 삭제
    pub async fn delete_key(&self, key: &str) -> Result<()> {
        let mut conn_guard = self.connection.lock().await;
        let conn = conn_guard
            .as_mut()
            .context("Not connected to Redis")?;
        
        let _deleted_count: i32 = conn.del(key)
            .await
            .context("Failed to delete key")?;
        
        Ok(())
    }
    // ...existing code...

    /// 키 존재 여부 확인
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn_guard = self.connection.lock().await;
        let conn = conn_guard
            .as_mut()
            .context("Not connected to Redis")?;
        
        let exists: bool = conn
            .exists(key)
            .await
            .context("Failed to check key existence")?;
        
        Ok(exists)
    }

    /// 키의 TTL 조회 (초 단위)
    pub async fn ttl(&self, key: &str) -> Result<i64> {
        let mut conn_guard = self.connection.lock().await;
        let conn = conn_guard
            .as_mut()
            .context("Not connected to Redis")?;
        
        let ttl: i64 = conn
            .ttl(key)
            .await
            .context("Failed to get TTL")?;
        
        Ok(ttl)
    }

    /// 키에 만료 시간 설정 (초 단위)
    pub async fn expire(&self, key: &str, seconds: i64) -> Result<()> {
        let mut conn_guard = self.connection.lock().await;
        let conn = conn_guard
            .as_mut()
            .context("Not connected to Redis")?;
        
        let _: () = conn.expire(key, seconds)
            .await
            .context("Failed to set expiration")?;
        
        Ok(())
    }

    /// Redis 정보 조회
    pub async fn info(&self) -> Result<String> {
        let mut conn_guard = self.connection.lock().await;
        let conn = conn_guard
            .as_mut()
            .context("Not connected to Redis")?;
        
        redis::cmd("INFO")
            .query_async(conn)
            .await
            .context("Failed to get INFO")
    }

    /// 데이터베이스 크기 조회
    pub async fn dbsize(&self) -> Result<usize> {
        let mut conn_guard = self.connection.lock().await;
        let conn = conn_guard
            .as_mut()
            .context("Not connected to Redis")?;
        
        redis::cmd("DBSIZE")
            .query_async(conn)
            .await
            .context("Failed to get DBSIZE")
    }
}

impl Clone for RedisManager {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            connection: Arc::clone(&self.connection),
        }
    }
}
