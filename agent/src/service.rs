#[cfg(windows)]
pub mod windows {
    use std::ffi::OsString;
    use std::sync::mpsc;
    use std::time::Duration;
    use windows_service::{
        define_windows_service,
        service::{
            ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
            ServiceType,
        },
        service_control_handler::{self, ServiceControlHandlerResult},
        service_dispatcher,
    };

    const SERVICE_NAME: &str = "EndpointAgent";
    const SERVICE_TYPE: ServiceType = ServiceType::OWN_PROCESS;

    pub fn run_as_service() -> Result<(), windows_service::Error> {
        service_dispatcher::start(SERVICE_NAME, ffi_service_main)
    }

    define_windows_service!(ffi_service_main, service_main);

    fn service_main(arguments: Vec<OsString>) {
        if let Err(e) = run_service(arguments) {
            tracing::error!("Service error: {:?}", e);
        }
    }

    fn run_service(_arguments: Vec<OsString>) -> Result<(), windows_service::Error> {
        let (shutdown_tx, shutdown_rx) = mpsc::channel();

        let event_handler = move |control_event| -> ServiceControlHandlerResult {
            match control_event {
                ServiceControl::Stop | ServiceControl::Shutdown => {
                    shutdown_tx.send(()).ok();
                    ServiceControlHandlerResult::NoError
                }
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                _ => ServiceControlHandlerResult::NotImplemented,
            }
        };

        let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

        // Report service as running
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        // Run the agent in a separate thread
        let agent_handle = std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(crate::run_agent())
        });

        // Wait for shutdown signal
        loop {
            match shutdown_rx.recv_timeout(Duration::from_secs(1)) {
                Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Check if agent thread is still running
                    if agent_handle.is_finished() {
                        break;
                    }
                }
            }
        }

        // Report service as stopped
        status_handle.set_service_status(ServiceStatus {
            service_type: SERVICE_TYPE,
            current_state: ServiceState::Stopped,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;

        Ok(())
    }

    pub fn install_service() -> anyhow::Result<()> {
        use std::ffi::OsStr;
        use windows_service::{
            service::{ServiceAccess, ServiceErrorControl, ServiceInfo, ServiceStartType},
            service_manager::{ServiceManager, ServiceManagerAccess},
        };

        let manager =
            ServiceManager::local_computer(None::<&OsStr>, ServiceManagerAccess::CREATE_SERVICE)?;

        let service_binary_path = std::env::current_exe()?;

        let service_info = ServiceInfo {
            name: OsString::from(SERVICE_NAME),
            display_name: OsString::from("Endpoint Assessment Agent"),
            service_type: SERVICE_TYPE,
            start_type: ServiceStartType::AutoStart,
            error_control: ServiceErrorControl::Normal,
            executable_path: service_binary_path,
            launch_arguments: vec![OsString::from("--service")],
            dependencies: vec![],
            account_name: None, // LocalSystem
            account_password: None,
        };

        let _service = manager.create_service(&service_info, ServiceAccess::CHANGE_CONFIG)?;

        println!("Service '{}' installed successfully.", SERVICE_NAME);
        println!();
        println!("Next steps:");
        println!("1. Configure the service by setting environment variables:");
        println!("   setx /M SERVER_URL \"http://your-server:8080\"");
        println!("   setx /M AGENT_SECRET \"your-agent-secret\"");
        println!();
        println!("2. Start the service:");
        println!("   sc start {}", SERVICE_NAME);
        println!();
        println!("Or use Services management console (services.msc)");

        Ok(())
    }

    pub fn uninstall_service() -> anyhow::Result<()> {
        use std::ffi::OsStr;
        use windows_service::{
            service::ServiceAccess,
            service_manager::{ServiceManager, ServiceManagerAccess},
        };

        let manager =
            ServiceManager::local_computer(None::<&OsStr>, ServiceManagerAccess::CONNECT)?;

        let service = manager.open_service(
            SERVICE_NAME,
            ServiceAccess::DELETE | ServiceAccess::STOP | ServiceAccess::QUERY_STATUS,
        )?;

        // Stop service if running
        let _ = service.stop();

        // Wait a bit for service to stop
        std::thread::sleep(Duration::from_secs(2));

        service.delete()?;

        println!("Service '{}' uninstalled successfully.", SERVICE_NAME);

        Ok(())
    }
}
