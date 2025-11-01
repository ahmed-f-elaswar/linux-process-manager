// REST API module for programmatic access to process manager
// Provides HTTP endpoints for querying processes, sending signals, and accessing historical data

use crate::process::{ProcessManager, ProcessFilter, SortColumn, ProcessInfo};
use crate::history::HistoryManager;
use actix_web::{web, App, HttpServer, HttpResponse, Responder, middleware};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::interval;
use chrono::{DateTime, Utc};
use tracing::{debug, info, warn, error};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiProcessInfo {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
    pub user: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub memory_percent: f32,
    pub state: String,
    pub command: String,
    pub start_time: u64,
    pub network_connections: Option<usize>,
    pub is_container: bool,
    pub container_id: Option<String>,
    pub gpu_memory: Option<u64>,
}

impl From<&ProcessInfo> for ApiProcessInfo {
    fn from(p: &ProcessInfo) -> Self {
        ApiProcessInfo {
            pid: p.pid,
            ppid: p.ppid,
            name: p.name.clone(),
            user: p.user.clone(),
            cpu_usage: p.cpu_usage,
            memory_usage: p.memory_usage,
            memory_percent: p.memory_percent,
            state: p.status.clone(),
            command: p.command.clone(),
            start_time: p.start_time,
            network_connections: p.network_connections,
            is_container: p.is_container,
            container_id: p.container_id.clone(),
            gpu_memory: p.gpu_memory,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SystemInfoResponse {
    pub cpu_count: usize,
    pub total_memory: u64,
    pub used_memory: u64,
    pub total_swap: u64,
    pub used_swap: u64,
    pub uptime: u64,
    pub load_average: LoadAverage,
}

#[derive(Debug, Serialize)]
pub struct LoadAverage {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

#[derive(Debug, Serialize)]
pub struct ProcessListResponse {
    pub processes: Vec<ApiProcessInfo>,
    pub total: usize,
    pub filtered: usize,
}

#[derive(Debug, Deserialize)]
pub struct ProcessQuery {
    pub sort_by: Option<String>,
    pub ascending: Option<bool>,
    pub user: Option<String>,
    pub name: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct KillRequest {
    pub pid: u32,
    pub signal: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct KillResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub pid: Option<u32>,
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

pub struct AppState {
    pub process_manager: Arc<Mutex<ProcessManager>>,
    pub history_manager: Option<Arc<Mutex<HistoryManager>>>,
}

// API Endpoints

/// GET /api/processes - List all processes
async fn get_processes(
    state: web::Data<AppState>,
    query: web::Query<ProcessQuery>,
) -> impl Responder {
    debug!("API: GET /processes - query params: {:?}", query);
    
    let mut pm = state.process_manager.lock().unwrap();
    
    // Refresh process data
    if let Err(e) = pm.refresh() {
        error!("API: Failed to refresh processes: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to refresh processes: {}", e)
        }));
    }
    
    info!("API: Returning {} processes", pm.get_processes().len());
    
    // Parse sort column
    let sort_column = match query.sort_by.as_deref() {
        Some("pid") => SortColumn::Pid,
        Some("name") => SortColumn::Name,
        Some("user") => SortColumn::User,
        Some("cpu") => SortColumn::CpuUsage,
        Some("memory") => SortColumn::MemoryUsage,
        Some("start_time") => SortColumn::StartTime,
        _ => SortColumn::CpuUsage,
    };
    
    let ascending = query.ascending.unwrap_or(false);
    
    // Sort processes
    let processes = pm.sort_processes(sort_column, ascending);
    
    // Apply filters
    let mut filter = ProcessFilter::new();
    if let Some(ref user) = query.user {
        filter.username = Some(user.clone());
    }
    if let Some(ref name) = query.name {
        if let Ok(regex) = regex::Regex::new(name) {
            filter.name_pattern = Some(regex);
        }
    }
    
    let filtered_processes: Vec<ApiProcessInfo> = processes
        .iter()
        .filter(|p| filter.matches(p))
        .take(query.limit.unwrap_or(1000))
        .map(|p| ApiProcessInfo::from(p))
        .collect();
    
    let total = processes.len();
    let filtered = filtered_processes.len();
    
    HttpResponse::Ok().json(ProcessListResponse {
        processes: filtered_processes,
        total,
        filtered,
    })
}

/// GET /api/processes/:pid - Get specific process info
async fn get_process(
    state: web::Data<AppState>,
    pid: web::Path<u32>,
) -> impl Responder {
    let mut pm = state.process_manager.lock().unwrap();
    
    if let Err(e) = pm.refresh() {
        return HttpResponse::InternalServerError().json(serde_json::json!({
            "error": format!("Failed to refresh processes: {}", e)
        }));
    }
    
    if let Some(process) = pm.get_process(*pid) {
        HttpResponse::Ok().json(ApiProcessInfo::from(process))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "error": format!("Process {} not found", pid)
        }))
    }
}

/// POST /api/processes/kill - Kill a process
async fn kill_process(
    state: web::Data<AppState>,
    req: web::Json<KillRequest>,
) -> impl Responder {
    let signal = req.signal.unwrap_or(15); // Default to SIGTERM
    info!("API: POST /kill - PID: {}, Signal: {}", req.pid, signal);
    
    let pm = state.process_manager.lock().unwrap();
    
    match pm.kill_process(req.pid, signal) {
        Ok(()) => {
            info!("API: Successfully sent signal {} to PID {}", signal, req.pid);
            HttpResponse::Ok().json(KillResponse {
                success: true,
                message: format!("Sent signal {} to process {}", signal, req.pid),
            })
        },
        Err(e) => {
            warn!("API: Failed to send signal {} to PID {}: {}", signal, req.pid, e);
            HttpResponse::InternalServerError().json(KillResponse {
                success: false,
                message: format!("Failed to kill process: {}", e),
            })
        },
    }
}

/// GET /api/system - Get system information
async fn get_system_info(state: web::Data<AppState>) -> impl Responder {
    let pm = state.process_manager.lock().unwrap();
    let sys_info = pm.get_system_info();
    
    HttpResponse::Ok().json(SystemInfoResponse {
        cpu_count: sys_info.cpu_count,
        total_memory: sys_info.total_memory,
        used_memory: sys_info.used_memory,
        total_swap: sys_info.total_swap,
        used_swap: sys_info.used_swap,
        uptime: sys_info.uptime,
        load_average: LoadAverage {
            one: sys_info.load_average.one,
            five: sys_info.load_average.five,
            fifteen: sys_info.load_average.fifteen,
        },
    })
}

/// GET /api/history/processes - Get process history
async fn get_process_history(
    state: web::Data<AppState>,
    query: web::Query<HistoryQuery>,
) -> impl Responder {
    if let Some(ref history_manager) = state.history_manager {
        let hm = history_manager.lock().unwrap();
        
        let start = query.start.unwrap_or_else(|| Utc::now() - chrono::Duration::hours(1));
        let end = query.end.unwrap_or_else(|| Utc::now());
        
        if let Some(pid) = query.pid {
            match hm.get_process_history(pid, start, end) {
                Ok(history) => {
                    let limited_history: Vec<_> = history
                        .into_iter()
                        .take(query.limit.unwrap_or(1000))
                        .collect();
                    return HttpResponse::Ok().json(limited_history);
                }
                Err(e) => {
                    return HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": format!("Failed to fetch history: {}", e)
                    }));
                }
            }
        } else {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": "pid parameter is required"
            }));
        }
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "History manager not enabled"
        }))
    }
}

/// GET /api/history/top-cpu - Get top CPU consumers
async fn get_top_cpu_consumers(
    state: web::Data<AppState>,
    query: web::Query<HistoryQuery>,
) -> impl Responder {
    if let Some(ref history_manager) = state.history_manager {
        let hm = history_manager.lock().unwrap();
        
        let start = query.start.unwrap_or_else(|| Utc::now() - chrono::Duration::hours(1));
        let end = query.end.unwrap_or_else(|| Utc::now());
        let limit = query.limit.unwrap_or(10);
        
        match hm.get_top_cpu_consumers(start, end, limit) {
            Ok(top_processes) => HttpResponse::Ok().json(top_processes),
            Err(e) => {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Failed to fetch top CPU consumers: {}", e)
                }))
            }
        }
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "error": "History manager not enabled"
        }))
    }
}

/// GET /api/health - Health check endpoint
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now().to_rfc3339(),
    }))
}

// Background task to record historical data
async fn record_history_task(state: Arc<AppState>) {
    let mut interval = interval(Duration::from_secs(60)); // Record every minute
    
    loop {
        interval.tick().await;
        
        if let Some(ref history_manager) = state.history_manager {
            let mut pm = state.process_manager.lock().unwrap();
            
            if let Err(e) = pm.refresh() {
                eprintln!("Failed to refresh processes for history: {}", e);
                continue;
            }
            
            let processes_refs = pm.get_processes();
            // Clone to owned values for history recording
            let processes: Vec<ProcessInfo> = processes_refs.iter().map(|p| (*p).clone()).collect();
            let system_info = pm.get_system_info();
            
            let hm = history_manager.lock().unwrap();
            
            if let Err(e) = hm.record_processes(&processes) {
                eprintln!("Failed to record process history: {}", e);
            }
            
            if let Err(e) = hm.record_system_stats(
                system_info.cpu_count,
                (system_info.load_average.one, system_info.load_average.five, system_info.load_average.fifteen),
                system_info.total_memory,
                system_info.used_memory,
                system_info.total_swap,
                system_info.used_swap,
                system_info.uptime,
            ) {
                eprintln!("Failed to record system history: {}", e);
            }
        }
    }
}

/// Start the REST API server
pub async fn start_api_server(
    bind_address: &str,
    process_manager: ProcessManager,
    history_db_path: Option<String>,
) -> std::io::Result<()> {
    let pm = Arc::new(Mutex::new(process_manager));
    
    let history_manager = if let Some(db_path) = history_db_path {
        match HistoryManager::new(&db_path) {
            Ok(hm) => Some(Arc::new(Mutex::new(hm))),
            Err(e) => {
                eprintln!("Failed to initialize history manager: {}", e);
                None
            }
        }
    } else {
        None
    };
    
    let app_state = Arc::new(AppState {
        process_manager: pm,
        history_manager: history_manager.clone(),
    });
    
    // Start background history recording task
    if history_manager.is_some() {
        let state_clone = app_state.clone();
        tokio::spawn(async move {
            record_history_task(state_clone).await;
        });
    }
    
    println!("Starting REST API server on {}", bind_address);
    
    let app_state_data = web::Data::new(AppState {
        process_manager: app_state.process_manager.clone(),
        history_manager: app_state.history_manager.clone(),
    });
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();
        
        App::new()
            .app_data(app_state_data.clone())
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .route("/api/health", web::get().to(health_check))
            .route("/api/processes", web::get().to(get_processes))
            .route("/api/processes/{pid}", web::get().to(get_process))
            .route("/api/processes/kill", web::post().to(kill_process))
            .route("/api/system", web::get().to(get_system_info))
            .route("/api/history/processes", web::get().to(get_process_history))
            .route("/api/history/top-cpu", web::get().to(get_top_cpu_consumers))
    })
    .bind(bind_address)?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_process_info_conversion() {
        use std::time::Duration;
        
        let process = ProcessInfo {
            pid: 1234,
            ppid: 1,
            name: "test".to_string(),
            user: "root".to_string(),
            cpu_usage: 10.5,
            memory_usage: 1024,
            memory_percent: 5.0,
            status: "R".to_string(),
            command: "test command".to_string(),
            start_time: 123456,
            running_time: Duration::from_secs(3600),
            uid: 0,
            gid: 0,
            threads: 1,
            priority: 20,
            nice: 0,
            network_connections: Some(5),
            is_container: false,
            container_id: None,
            cgroup_memory_limit: None,
            gpu_memory: None,
        };
        
        let api_info = ApiProcessInfo::from(&process);
        assert_eq!(api_info.pid, 1234);
        assert_eq!(api_info.name, "test");
    }
}
