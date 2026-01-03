use crate::libs::bulk_logger::{BulkLoggerHandler, LoggerArgs};
use crate::libs::find_server::get_server_pid;
use contract::{LogLevel, LogPacket, ManagerCommand, ManagerResponse, topics};
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS};
use sea_orm::{Database, DatabaseConnection};
use std::sync::Arc;
use std::time::Duration;
use sysinfo::System;
use tokio::sync::RwLock;
use tokio::time::interval;

pub use contract::config::ManagerConfig;

pub struct ManagerService {
  config: ManagerConfig,
  logger: Arc<BulkLoggerHandler>,
  db: Arc<DatabaseConnection>,
  mqtt_client: Arc<AsyncClient>,
  system_info: Arc<RwLock<System>>,
}

impl ManagerService {
  pub async fn new(config: ManagerConfig) -> Result<Self, Box<dyn std::error::Error>> {
    // Initialize logger
    let logger = BulkLoggerHandler::new(
      LoggerArgs::Url(config.common.redis_url.clone()),
      config.log_batch_size,
      config.log_flush_interval_secs,
    )
    .await
    .ok_or("Failed to initialize BulkLoggerHandler")?;
    
    let logger = Arc::new(logger);
    
    // Log initialization start
    logger
    .log(
      LogPacket::new(LogLevel::Info, "Initializing ManagerService...")
      .with_source("manager"),
    )
    .await;
    
    // Initialize database connection
    let db = Database::connect(&config.common.database_url)
    .await
    .map_err(|e| format!("Failed to connect to database: {}", e))?;
    let db = Arc::new(db);
    
    // Initialize MQTT client
    let mut mqtt_options = MqttOptions::new(
      &config.mqtt_client_id,
      &config.common.mqtt_host,
      config.common.mqtt_port,
    );
    mqtt_options.set_keep_alive(Duration::from_secs(30));
    
    let (mqtt_client, event_loop) = AsyncClient::new(mqtt_options, 10);
    let mqtt_client = Arc::new(mqtt_client);
    
    // Spawn MQTT event loop handler
    let logger_clone = Arc::clone(&logger);
    tokio::spawn(async move {
      Self::handle_mqtt_events(event_loop, logger_clone).await;
    });
    
    // Initialize system info
    let mut sys = System::new_all();
    sys.refresh_all();
    let system_info = Arc::new(RwLock::new(sys));
    
    logger
    .log(
      LogPacket::new(LogLevel::Info, "ManagerService initialized successfully")
      .with_source("manager"),
    )
    .await;
    
    Ok(Self {
      config,
      logger,
      db,
      mqtt_client,
      system_info,
    })
  }
  
  async fn handle_mqtt_events(mut event_loop: EventLoop, logger: Arc<BulkLoggerHandler>) {
    loop {
      match event_loop.poll().await {
        Ok(event) => {
          if let Event::Incoming(Packet::Publish(publish)) = event {
            let topic = publish.topic.clone();
            let payload = String::from_utf8_lossy(&publish.payload).into_owned();
            
            // Handle manager commands
            if topic == topics::MANAGER_COMMAND {
              if let Ok(cmd) = serde_json::from_str::<ManagerCommand>(&payload) {
                logger
                .log(
                  LogPacket::new(
                    LogLevel::Info,
                    format!("Received command: {:?}", cmd),
                  )
                  .with_source("manager"),
                )
                .await;
                // TODO: Handle command execution
              }
            } else {
              logger
              .log(
                LogPacket::new(
                  LogLevel::Info,
                  format!(
                    "MQTT message received - Topic: {}, Payload: {}",
                    topic, payload
                  ),
                )
                .with_source("manager"),
              )
              .await;
            }
          }
        }
        Err(e) => {
          logger
          .log(
            LogPacket::new(LogLevel::Error, format!("MQTT error: {}", e))
            .with_source("manager"),
          )
          .await;
          tokio::time::sleep(Duration::from_secs(5)).await;
        }
      }
    }
  }
  
  pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
    self.logger
    .log(
      LogPacket::new(LogLevel::Info, "Starting ManagerService...")
      .with_source("manager"),
    )
    .await;
    
    // Subscribe to MQTT topics using contract definitions
    self.mqtt_client
    .subscribe(topics::MANAGER_COMMAND, QoS::AtLeastOnce)
    .await?;
    
    self.mqtt_client
    .subscribe(topics::MANAGER_HEALTH_REPORT, QoS::AtLeastOnce)
    .await?;
    
    // Start health monitoring
    let logger_clone = Arc::clone(&self.logger);
    let system_info_clone = Arc::clone(&self.system_info);
    let mqtt_client_clone = Arc::clone(&self.mqtt_client);
    let health_check_interval = self.config.health_check_interval_secs;
    
    tokio::spawn(async move {
      Self::health_monitoring_loop(
        logger_clone,
        system_info_clone,
        mqtt_client_clone,
        health_check_interval,
      )
      .await;
    });
    
    // Start process monitoring if configured
    if let Some(ref process_name) = self.config.monitored_process_name {
      let process_name = process_name.clone();
      let logger_clone = Arc::clone(&self.logger);
      let mqtt_client_clone = Arc::clone(&self.mqtt_client);
      
      tokio::spawn(async move {
        Self::process_monitoring_loop(logger_clone, mqtt_client_clone, process_name).await;
      });
    }
    
    self.logger
    .log(
      LogPacket::new(LogLevel::Info, "ManagerService started successfully")
      .with_source("manager"),
    )
    .await;
    
    Ok(())
  }
  
  async fn health_monitoring_loop(
    logger: Arc<BulkLoggerHandler>,
    system_info: Arc<RwLock<System>>,
    mqtt_client: Arc<AsyncClient>,
    interval_secs: u64,
  ) {
    let mut interval = interval(Duration::from_secs(interval_secs));
    
    loop {
      interval.tick().await;
      
      let mut sys = system_info.write().await;
      sys.refresh_all();
      
      let cpu_usage = sys.global_cpu_usage();
      let total_memory = sys.total_memory();
      let used_memory = sys.used_memory();
      let memory_usage_percent = (used_memory as f64 / total_memory as f64) * 100.0;
      
      // Create ManagerResponse for health report
      let health_response = ManagerResponse::HealthReport {
        cpu_usage,
        memory_usage_percent,
        total_memory_mb: total_memory / 1024 / 1024,
        used_memory_mb: used_memory / 1024 / 1024,
        timestamp: chrono::Utc::now().timestamp() as u64,
      };
      
      let health_msg = format!(
        "CPU: {:.2}%, Memory: {:.2}% ({}/{} MB)",
        cpu_usage,
        memory_usage_percent,
        used_memory / 1024 / 1024,
        total_memory / 1024 / 1024
      );
      
      logger
      .log(
        LogPacket::new(LogLevel::Info, format!("System Health: {}", health_msg))
        .with_source("manager"),
      )
      .await;
      
      // Publish structured response
      let json = serde_json::to_string(&health_response).unwrap();
      if let Err(e) = mqtt_client
      .publish(topics::MANAGER_HEALTH_REPORT, QoS::AtLeastOnce, false, json)
      .await
      {
        logger
        .log(
          LogPacket::new(
            LogLevel::Error,
            format!("Failed to publish health report: {}", e),
          )
          .with_source("manager"),
        )
        .await;
      }
    }
  }
  
  async fn process_monitoring_loop(
    logger: Arc<BulkLoggerHandler>,
    mqtt_client: Arc<AsyncClient>,
    process_name: String,
  ) {
    let mut interval = interval(Duration::from_secs(10));
    
    loop {
      interval.tick().await;
      
      match get_server_pid(&process_name) {
        Some(pid) => {
          let status = ManagerResponse::ProcessStatus {
            process_name: process_name.clone(),
            is_running: true,
            pid: Some(pid.as_u32()),
            timestamp: chrono::Utc::now().timestamp() as u64,
          };
          
          logger
          .log(
            LogPacket::new(
              LogLevel::Info,
              format!("Process '{}' is running with PID: {:?}", process_name, pid),
            )
            .with_source("manager"),
          )
          .await;
          
          let json = serde_json::to_string(&status).unwrap();
          let _ = mqtt_client
          .publish(topics::MANAGER_PROCESS_STATUS, QoS::AtLeastOnce, false, json)
          .await;
        }
        None => {
          let status = ManagerResponse::ProcessStatus {
            process_name: process_name.clone(),
            is_running: false,
            pid: None,
            timestamp: chrono::Utc::now().timestamp() as u64,
          };
          
          logger
          .log(
            LogPacket::new(
              LogLevel::Warn,
              format!("Process '{}' not found!", process_name),
            )
            .with_source("manager"),
          )
          .await;
          
          let json = serde_json::to_string(&status).unwrap();
          let _ = mqtt_client
          .publish(topics::MANAGER_PROCESS_ALERT, QoS::AtLeastOnce, false, json)
          .await;
        }
      }
    }
  }
  
  pub fn logger(&self) -> &Arc<BulkLoggerHandler> {
    &self.logger
  }
  
  pub fn db(&self) -> &Arc<DatabaseConnection> {
    &self.db
  }
  
  pub fn mqtt_client(&self) -> &Arc<AsyncClient> {
    &self.mqtt_client
  }
}
