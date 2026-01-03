use contract::LogPacket;
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::interval;

// Re-export LogPacket from contract
pub use contract::LogPacket as BulkLogPacket;

pub enum LoggerArgs {
    RedisClient(redis::Client),
    Url(String),
}

struct BulkLogger {
    redis_client: redis::Client,
    connection: Option<MultiplexedConnection>,
}

impl BulkLogger {
    pub async fn new(arg: LoggerArgs) -> Option<Self> {
        let redis_client = match arg {
            LoggerArgs::RedisClient(client) => client,
            LoggerArgs::Url(url) => match redis::Client::open(url) {
                Ok(client) => client,
                Err(ref e) => {
                    eprintln!("Failed to create Redis client from URL: {}", e);
                    return None;
                }
            },
        };

        let connection = match redis_client.get_multiplexed_async_connection().await {
            Ok(conn) => Some(conn),
            Err(ref e) => {
                eprintln!("Failed to connect to Redis: {}", e);
                None
            }
        };

        if connection.is_none() {
            eprintln!("Warning: BulkLogger initialized without a valid Redis connection.");
        }

        Some(BulkLogger {
            redis_client,
            connection,
        })
    }

    pub async fn push(&mut self, packet: BulkLogPacket) {
        if let Some(ref mut conn) = self.connection {
            let serialized = match serde_json::to_string(&packet) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to serialize log packet: {}", e);
                    return;
                }
            };

            if let Err(e) = conn.lpush::<&str, String, ()>("logs", serialized).await {
                eprintln!("Failed to push to Redis: {}", e);
            }
        }
    }

    pub async fn bulk_push(&mut self, packets: Vec<BulkLogPacket>) {
        if packets.is_empty() {
            return;
        }

        if let Some(ref mut conn) = self.connection {
            let serialized: Vec<String> = packets
                .into_iter()
                .filter_map(|p| serde_json::to_string(&p).ok())
                .collect();

            if !serialized.is_empty() {
                if let Err(e) = conn.lpush::<&str, Vec<String>, ()>("logs", serialized).await {
                    eprintln!("Failed to bulk push to Redis: {}", e);
                }
            }
        }
    }
}

pub struct BulkLoggerHandler {
    tx: mpsc::Sender<BulkLogPacket>,
}

impl BulkLoggerHandler {
    pub async fn new(
        logger_args: LoggerArgs,
        batch_size: usize,
        flush_interval_secs: u64,
    ) -> Option<Self> {
        let logger = BulkLogger::new(logger_args).await?;
        let (tx, rx) = mpsc::channel::<BulkLogPacket>(1000);

        tokio::spawn(async move {
            Self::process_logs(logger, rx, batch_size, flush_interval_secs).await;
        });

        Some(Self { tx })
    }

    async fn process_logs(
        mut logger: BulkLogger,
        mut rx: mpsc::Receiver<BulkLogPacket>,
        batch_size: usize,
        flush_interval_secs: u64,
    ) {
        let mut buffer = Vec::with_capacity(batch_size);
        let mut flush_timer = interval(Duration::from_secs(flush_interval_secs));

        loop {
            tokio::select! {
                Some(packet) = rx.recv() => {
                    buffer.push(packet);
                    if buffer.len() >= batch_size {
                        logger.bulk_push(buffer.drain(..).collect()).await;
                    }
                }
                _ = flush_timer.tick() => {
                    if !buffer.is_empty() {
                        logger.bulk_push(buffer.drain(..).collect()).await;
                    }
                }
            }
        }
    }

    pub async fn log(&self, packet: BulkLogPacket) {
        if let Err(e) = self.tx.send(packet).await {
            eprintln!("Failed to send log packet to channel: {}", e);
        }
    }

    pub fn log_sync(&self, packet: BulkLogPacket) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            if let Err(e) = tx.send(packet).await {
                eprintln!("Failed to send log packet to channel: {}", e);
            }
        });
    }
}
