use chrono::Utc;
use common::{ProcessInfo, SoftwareInfo, SystemSnapshotData};
use sysinfo::{Disks, Networks, System};
use std::net::TcpListener;

pub struct SystemCollector {
    system: System,
}

impl SystemCollector {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }

    pub fn refresh(&mut self) {
        self.system.refresh_all();
    }

    pub fn get_hostname(&self) -> String {
        System::host_name().unwrap_or_else(|| "unknown".to_string())
    }

    pub fn get_os(&self) -> String {
        System::name().unwrap_or_else(|| "unknown".to_string())
    }

    pub fn get_os_version(&self) -> String {
        System::os_version().unwrap_or_else(|| "unknown".to_string())
    }

    pub fn get_ip_addresses(&self) -> Vec<String> {
        let networks = Networks::new_with_refreshed_list();
        let mut ips = Vec::new();

        for (_name, data) in &networks {
            for ip in data.ip_networks() {
                let addr = ip.addr.to_string();
                // Filter out loopback and link-local addresses
                if !addr.starts_with("127.") && !addr.starts_with("::1") && !addr.starts_with("fe80") {
                    ips.push(addr);
                }
            }
        }

        ips.sort();
        ips.dedup();
        ips
    }

    pub fn collect_snapshot(&mut self) -> SystemSnapshotData {
        self.refresh();

        let cpu_usage = self.system.global_cpu_usage();
        let memory_total = self.system.total_memory();
        let memory_used = self.system.used_memory();

        let disks = Disks::new_with_refreshed_list();
        let (disk_total, disk_used) = disks.iter().fold((0u64, 0u64), |(total, used), disk| {
            (total + disk.total_space(), used + (disk.total_space() - disk.available_space()))
        });

        let processes: Vec<ProcessInfo> = self
            .system
            .processes()
            .iter()
            .take(100) // Limit to top 100 processes
            .map(|(pid, process)| ProcessInfo {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().to_string(),
                cpu_usage: process.cpu_usage(),
                memory_bytes: process.memory(),
            })
            .collect();

        let open_ports = self.collect_open_ports();

        SystemSnapshotData {
            collected_at: Utc::now(),
            cpu_usage,
            memory_total,
            memory_used,
            disk_total,
            disk_used,
            processes,
            open_ports,
            installed_software: Vec::new(), // TODO: Implement per-platform
        }
    }

    fn collect_open_ports(&self) -> Vec<u16> {
        let mut ports = Vec::new();

        // Check common ports
        let common_ports = [22, 80, 443, 3306, 5432, 8080, 8443, 3000, 5000, 6379, 27017];

        for port in common_ports {
            if is_port_in_use(port) {
                ports.push(port);
            }
        }

        ports
    }
}

fn is_port_in_use(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_err()
}

impl Default for SystemCollector {
    fn default() -> Self {
        Self::new()
    }
}
