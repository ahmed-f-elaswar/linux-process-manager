//! # Smart Alerts and Notifications System
//! 
//! Provides real-time alerting for process events and anomalies with multiple
//! notification channels (email, webhook, desktop notifications).
//! 
//! ## Features
//! 
//! - **Rule-Based Alerts**: CPU, memory, process lifecycle events
//! - **Multiple Channels**: Email (SMTP), webhooks (HTTP POST), desktop notifications
//! - **Severity Levels**: Info, Warning, Critical
//! - **Cooldown Prevention**: Avoid alert storms
//! - **Process Filtering**: Alert on specific processes or patterns
//! - **Async Processing**: Non-blocking alert delivery
//! 
//! ## Alert Types
//! 
//! - High CPU usage (configurable threshold)
//! - High memory usage
//! - Process terminated/started
//! - Anomaly detection triggers
//! - Custom alerts
//! 
//! ## Example
//! 
//! ```rust,ignore
//! use process_manager::alerts::{AlertManager, AlertRule, AlertType, NotificationConfig};
//! 
//! # #[tokio::main]
//! # async fn main() {
//! let config = NotificationConfig::default();
//! let manager = AlertManager::new(config);
//! 
//! // Add rule: alert if any process uses >80% CPU
//! let rule = AlertRule {
//!     enabled: true,
//!     alert_type: AlertType::HighCpu,
//!     threshold: 80.0,
//!     duration_secs: 30,
//!     cooldown_secs: 300,
//!     process_filter: None,
//! };
//! manager.add_rule(rule).await;
//! # }
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, info, warn, error};

/// Types of alerts that can be triggered.
/// 
/// Each type corresponds to a specific monitoring condition or event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AlertType {
    HighCpu,
    HighMemory,
    ProcessTerminated,
    ProcessStarted,
    AnomalyDetected,
    Custom(String),
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Alert notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub process_name: String,
    pub pid: u32,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub value: Option<f64>,
    pub threshold: Option<f64>,
}

/// Alert rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub enabled: bool,
    pub alert_type: AlertType,
    pub threshold: f64,
    pub duration_secs: u64,
    pub cooldown_secs: u64,
    pub process_filter: Option<String>,
}

/// Notification channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub email: Option<EmailConfig>,
    pub webhook: Option<WebhookConfig>,
    pub desktop: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub enabled: bool,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
    pub to: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub enabled: bool,
    pub url: String,
    pub headers: HashMap<String, String>,
}

/// Alert manager to handle alert rules and notifications
pub struct AlertManager {
    rules: Vec<AlertRule>,
    #[allow(dead_code)]
    notification_config: NotificationConfig,
    alert_state: HashMap<(AlertType, u32), AlertState>,
    alert_tx: mpsc::Sender<Alert>,
}

struct AlertState {
    triggered: bool,
    first_seen: Instant,
    last_sent: Instant,
    count: u32,
}

impl AlertManager {
    pub fn new(
        rules: Vec<AlertRule>,
        notification_config: NotificationConfig,
    ) -> (Self, mpsc::Receiver<Alert>) {
        let (tx, rx) = mpsc::channel(100);
        
        let manager = Self {
            rules,
            notification_config,
            alert_state: HashMap::new(),
            alert_tx: tx,
        };
        
        (manager, rx)
    }
    
    /// Check process against alert rules
    pub async fn check_process(
        &mut self,
        pid: u32,
        name: &str,
        cpu_usage: f32,
        memory_percent: f32,
    ) -> Result<()> {
        debug!("Checking process {} (pid {}) against alert rules: cpu={:.2}%, mem={:.2}%", 
               name, pid, cpu_usage, memory_percent);
        
        // Collect rules to check to avoid borrow issues
        let checks: Vec<_> = self.rules.iter()
            .filter(|rule| rule.enabled)
            .filter_map(|rule| {
                // Check process filter
                if let Some(ref filter) = rule.process_filter {
                    if !name.contains(filter) {
                        return None;
                    }
                }
                
                let (triggered, value) = match rule.alert_type {
                    AlertType::HighCpu => (cpu_usage as f64 > rule.threshold, cpu_usage as f64),
                    AlertType::HighMemory => (memory_percent as f64 > rule.threshold, memory_percent as f64),
                    _ => return None,
                };
                
                Some((rule.clone(), triggered, value))
            })
            .collect();
        
        for (rule, triggered, value) in checks {
            if triggered {
                self.handle_trigger(&rule, pid, name, value).await?;
            } else {
                self.handle_clear(&rule, pid).await;
            }
        }
        
        Ok(())
    }
    
    async fn handle_trigger(
        &mut self,
        rule: &AlertRule,
        pid: u32,
        name: &str,
        value: f64,
    ) -> Result<()> {
        let key = (rule.alert_type.clone(), pid);
        let now = Instant::now();
        
        let state = self.alert_state.entry(key.clone()).or_insert(AlertState {
            triggered: false,
            first_seen: now,
            last_sent: now - Duration::from_secs(rule.cooldown_secs + 1),
            count: 0,
        });
        
        // Check if condition has persisted long enough
        if now.duration_since(state.first_seen).as_secs() < rule.duration_secs {
            return Ok(());
        }
        
        // Check cooldown
        if now.duration_since(state.last_sent).as_secs() < rule.cooldown_secs {
            return Ok(());
        }
        
        // Send alert
        let severity = if value > rule.threshold * 1.5 {
            AlertSeverity::Critical
        } else {
            AlertSeverity::Warning
        };
        
        let alert = Alert {
            alert_type: rule.alert_type.clone(),
            severity: severity.clone(),
            process_name: name.to_string(),
            pid,
            message: format!(
                "Process '{}' (PID: {}) exceeded {} threshold: {:.2} > {:.2}",
                name, pid,
                match rule.alert_type {
                    AlertType::HighCpu => "CPU",
                    AlertType::HighMemory => "memory",
                    _ => "unknown",
                },
                value, rule.threshold
            ),
            timestamp: chrono::Utc::now(),
            value: Some(value),
            threshold: Some(rule.threshold),
        };
        
        match severity {
            AlertSeverity::Critical => error!("CRITICAL alert triggered for {} (pid {}): {:.2} > {:.2}", 
                                              name, pid, value, rule.threshold),
            AlertSeverity::Warning => warn!("Warning alert triggered for {} (pid {}): {:.2} > {:.2}", 
                                            name, pid, value, rule.threshold),
            AlertSeverity::Info => info!("Info alert triggered for {} (pid {}): {:.2} > {:.2}", 
                                        name, pid, value, rule.threshold),
        }
        
        self.alert_tx.send(alert).await?;
        state.last_sent = now;
        state.count += 1;
        state.triggered = true;
        
        Ok(())
    }
    
    async fn handle_clear(&mut self, rule: &AlertRule, pid: u32) {
        let key = (rule.alert_type.clone(), pid);
        if let Some(state) = self.alert_state.get_mut(&key) {
            if state.triggered {
                debug!("Alert cleared for pid {} (type: {:?})", pid, rule.alert_type);
            }
            state.triggered = false;
            state.first_seen = Instant::now();
        }
    }
    
    /// Process alert notifications in background
    pub async fn process_alerts(
        mut rx: mpsc::Receiver<Alert>,
        config: NotificationConfig,
    ) {
        info!("Starting alert notification processor");
        
        while let Some(alert) = rx.recv().await {
            info!("Processing {:?} alert: {}", alert.severity, alert.message);
            
            // Send desktop notification
            if config.desktop {
                debug!("Sending desktop notification for alert");
                if let Err(e) = send_desktop_notification(&alert) {
                    error!("Failed to send desktop notification: {}", e);
                } else {
                    debug!("Desktop notification sent successfully");
                }
            }
            
            // Send email
            if let Some(ref email_config) = config.email {
                if email_config.enabled {
                    debug!("Sending email notification to {:?}", email_config.to);
                    if let Err(e) = send_email_notification(&alert, email_config).await {
                        error!("Failed to send email notification: {}", e);
                    } else {
                        info!("Email notification sent successfully to {:?}", email_config.to);
                    }
                }
            }
            
            // Send webhook
            if let Some(ref webhook_config) = config.webhook {
                if webhook_config.enabled {
                    debug!("Sending webhook notification to {}", webhook_config.url);
                    if let Err(e) = send_webhook_notification(&alert, webhook_config).await {
                        error!("Failed to send webhook notification: {}", e);
                    } else {
                        info!("Webhook notification sent successfully to {}", webhook_config.url);
                    }
                }
            }
        }
        
        info!("Alert notification processor terminated");
    }
}

/// Send desktop notification
fn send_desktop_notification(alert: &Alert) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        use notify_rust::{Notification, Urgency};
        
        let urgency = match alert.severity {
            AlertSeverity::Critical => Urgency::Critical,
            AlertSeverity::Warning => Urgency::Normal,
            AlertSeverity::Info => Urgency::Low,
        };
        
        Notification::new()
            .summary("Process Manager Alert")
            .body(&alert.message)
            .urgency(urgency)
            .timeout(5000)
            .show()
            .context("Failed to show desktop notification")?;
    }
    
    Ok(())
}

/// Send email notification
async fn send_email_notification(alert: &Alert, config: &EmailConfig) -> Result<()> {
    use lettre::{
        Message, SmtpTransport, Transport,
        transport::smtp::authentication::Credentials,
    };
    
    let subject = format!(
        "[{}] Process Manager Alert: {}",
        match alert.severity {
            AlertSeverity::Critical => "CRITICAL",
            AlertSeverity::Warning => "WARNING",
            AlertSeverity::Info => "INFO",
        },
        alert.process_name
    );
    
    let body = format!(
        "Alert Details:\n\n\
         Type: {:?}\n\
         Process: {} (PID: {})\n\
         Message: {}\n\
         Time: {}\n\
         Value: {:?}\n\
         Threshold: {:?}\n",
        alert.alert_type,
        alert.process_name,
        alert.pid,
        alert.message,
        alert.timestamp,
        alert.value,
        alert.threshold,
    );
    
    for recipient in &config.to {
        let email = Message::builder()
            .from(config.from.parse()?)
            .to(recipient.parse()?)
            .subject(&subject)
            .body(body.clone())?;
        
        let creds = Credentials::new(
            config.username.clone(),
            config.password.clone(),
        );
        
        let mailer = SmtpTransport::relay(&config.smtp_server)?
            .credentials(creds)
            .port(config.smtp_port)
            .build();
        
        mailer.send(&email)?;
    }
    
    Ok(())
}

/// Send webhook notification
async fn send_webhook_notification(alert: &Alert, config: &WebhookConfig) -> Result<()> {
    let client = reqwest::Client::new();
    let mut request = client.post(&config.url)
        .json(&serde_json::json!({
            "alert_type": format!("{:?}", alert.alert_type),
            "severity": format!("{:?}", alert.severity),
            "process_name": alert.process_name,
            "pid": alert.pid,
            "message": alert.message,
            "timestamp": alert.timestamp.to_rfc3339(),
            "value": alert.value,
            "threshold": alert.threshold,
        }));
    
    for (key, value) in &config.headers {
        request = request.header(key, value);
    }
    
    let response = request.send().await?;
    
    if !response.status().is_success() {
        anyhow::bail!("Webhook returned error status: {}", response.status());
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_creation() {
        let alert = Alert {
            alert_type: AlertType::HighCpu,
            severity: AlertSeverity::Warning,
            process_name: "test".to_string(),
            pid: 1234,
            message: "High CPU usage".to_string(),
            timestamp: chrono::Utc::now(),
            value: Some(85.0),
            threshold: Some(80.0),
        };
        
        assert_eq!(alert.pid, 1234);
        assert_eq!(alert.process_name, "test");
    }

    #[test]
    fn test_alert_rule() {
        let rule = AlertRule {
            enabled: true,
            alert_type: AlertType::HighCpu,
            threshold: 80.0,
            duration_secs: 60,
            cooldown_secs: 300,
            process_filter: Some("nginx".to_string()),
        };
        
        assert!(rule.enabled);
        assert_eq!(rule.threshold, 80.0);
    }
}
