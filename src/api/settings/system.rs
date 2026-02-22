// 系统信息相关接口
use crate::common::state::AppState;
use crate::models::ApiResponse;
use axum::{extract::State, response::IntoResponse};
use serde::Serialize;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::process::Command;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, RefreshKind, System};

use super::common::respond_with_version;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub system_version: String,
    pub blockchain_version: String,
    pub os: String,
    pub kernel_version: String,
    pub cpu_desc: String,
    pub memory_desc: String,
    pub uptime: String,
    pub go_version: String,
    pub db_version: String,
    pub network_latency: String,
    pub disk_io: String,
    pub network_throughput: String,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub config_version: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemLogEntry {
    pub name: String,
    pub content: String,
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    format!("{}天 {}小时 {}分钟", days, hours, minutes)
}

fn run_command(cmd: &str) -> Option<String> {
    let output = Command::new("sh").arg("-c").arg(cmd).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn tail_lines(input: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = input.lines().collect();
    let start = lines.len().saturating_sub(max_lines);
    lines[start..].join("\n")
}

fn read_tail_content(path: &std::path::Path, max_bytes: u64) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let metadata = file.metadata().ok()?;
    let file_len = metadata.len();
    let start = file_len.saturating_sub(max_bytes);
    if file.seek(SeekFrom::Start(start)).is_err() {
        return None;
    }
    let mut buffer = Vec::new();
    if file.read_to_end(&mut buffer).is_err() {
        return None;
    }
    let content = String::from_utf8_lossy(&buffer).to_string();
    Some(content)
}

pub async fn get_system_info(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let general = state.general_settings.read().await.clone();

    let mut sys = System::new_with_specifics(
        RefreshKind::new()
            .with_memory(MemoryRefreshKind::new())
            .with_cpu(CpuRefreshKind::new().with_cpu_usage()),
    );
    sys.refresh_memory();
    sys.refresh_cpu();

    let os_name = System::name().unwrap_or_default();
    let os_version = System::os_version().unwrap_or_default();
    let os = if os_version.is_empty() {
        os_name
    } else {
        format!("{} {}", os_name, os_version)
    };

    let kernel_version = System::kernel_version().unwrap_or_default();

    let cpus = sys.cpus();
    let cpu_usage = if cpus.is_empty() {
        0.0
    } else {
        let total: f32 = cpus.iter().map(|c| c.cpu_usage()).sum();
        (total / cpus.len() as f32) as f64
    };

    let cpu_desc = if cpus.is_empty() {
        "Unknown CPU".to_string()
    } else {
        let brand = cpus[0].brand().to_string();
        let cores = cpus.len();
        format!("{} ({} 逻辑核)", brand, cores)
    };

    let total_mem = sys.total_memory();
    let used_mem = sys.used_memory();
    let memory_usage = if total_mem == 0 {
        0.0
    } else {
        (used_mem as f64) * 100.0 / (total_mem as f64)
    };
    let memory_desc = if total_mem == 0 {
        "未知内存".to_string()
    } else {
        let gb = total_mem as f64 / (1024.0 * 1024.0);
        format!("{:.0}MB", gb)
    };

    let uptime_secs = System::uptime();
    let uptime = format_uptime(uptime_secs);

    let disks = Disks::new_with_refreshed_list();
    let mut total_disk = 0u64;
    let mut used_disk = 0u64;
    for disk in disks.list() {
        let total = disk.total_space();
        let available = disk.available_space();
        total_disk += total;
        used_disk += total.saturating_sub(available);
    }
    let disk_usage = if total_disk == 0 {
        0.0
    } else {
        (used_disk as f64) * 100.0 / (total_disk as f64)
    };

    let system_version = format!("{} {}", general.system_name, general.version);

    let blockchain_version = run_command(
        "/home/hcp-dev/hcp-project/hcp-consensus-build/hcpd version 2>/dev/null | head -n 1",
    )
    .or_else(|| run_command("hcpd version 2>/dev/null | head -n 1"))
    .unwrap_or_else(|| "unknown".to_string());

    let go_version = run_command("go version 2>/dev/null | awk '{print $3 \" \" $4}'")
        .unwrap_or_else(|| "unknown".to_string());

    let db_version = run_command(
        "ldconfig -p 2>/dev/null | grep -m1 -E 'rocksdb|leveldb' | awk '{print $1}'",
    )
    .unwrap_or_else(|| "unknown".to_string());

    let network_latency = run_command(
        "ping -c 1 -W 1 8.8.8.8 2>/dev/null | awk -F'/' 'END{if($5!=\"\") print $5 \" ms\"}'",
    )
    .unwrap_or_else(|| "未知".to_string());

    let cpu_usage_cmd = run_command(
        "grep 'cpu ' /proc/stat | awk '{idle=$5; total=$2+$3+$4+$5+$6+$7+$8; if(total>0) print (total-idle)*100/total}'",
    )
    .and_then(|s| s.parse::<f64>().ok());

    let memory_usage_cmd = run_command(
        "free -m | awk '/Mem:/ {if($2>0) print $3*100/$2}'",
    )
    .and_then(|s| s.parse::<f64>().ok());

    let disk_usage_cmd = run_command(
        "df -P / | awk 'NR==2 {gsub(\"%\",\"\",$5); print $5}'",
    )
    .and_then(|s| s.parse::<f64>().ok());

    let cpu_usage_final = cpu_usage_cmd.unwrap_or(cpu_usage);
    let memory_usage_final = memory_usage_cmd.unwrap_or(memory_usage);
    let disk_usage_final = disk_usage_cmd.unwrap_or(disk_usage);

    let info = SystemInfo {
        system_version,
        blockchain_version,
        os,
        kernel_version,
        cpu_desc,
        memory_desc,
        uptime,
        go_version,
        db_version,
        network_latency,
        disk_io: "未知".to_string(),
        network_throughput: "未知".to_string(),
        cpu_usage: cpu_usage_final,
        memory_usage: memory_usage_final,
        disk_usage: disk_usage_final,
        config_version: state.config_version.load(Ordering::SeqCst),
    };

    respond_with_version(&state, ApiResponse::success(info), false)
}

pub async fn get_system_logs(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let log_dir = "/home/hcp-dev/hcp-project/logs";
    let mut entries = Vec::new();

    if let Ok(dir) = fs::read_dir(log_dir) {
        let mut files: Vec<_> = dir
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().map(|ext| ext == "log").unwrap_or(false))
            .collect();

        files.sort();

        for path in files {
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();
            if name.is_empty() {
                continue;
            }
            let content = read_tail_content(&path, 128 * 1024)
                .unwrap_or_else(|| "无法读取日志内容".to_string());
            entries.push(SystemLogEntry {
                name,
                content: tail_lines(&content, 200),
            });
        }
    }

    respond_with_version(&state, ApiResponse::success(entries), false)
}

pub async fn restart_system(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let script = "/home/hcp-dev/hcp-project/hcp/restart_hcp.sh";
    let log_path = "/home/hcp-dev/hcp-project/logs/restart.log";
    let cmd = format!("nohup bash {} > {} 2>&1 &", script, log_path);
    let status = Command::new("sh").arg("-c").arg(cmd).status();

    match status {
        Ok(_) => respond_with_version(
            &state,
            ApiResponse::success("重启任务已触发".to_string()),
            false,
        ),
        Err(_) => respond_with_version(&state, ApiResponse::error(500, "重启失败"), false),
    }
}
