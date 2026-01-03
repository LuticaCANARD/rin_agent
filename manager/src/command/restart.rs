use crate::libs::find_server::get_server_pid;
use sysinfo::System;

pub struct RestartCommand;

impl RestartCommand {
    pub async fn execute(process_name: &str) -> Result<String, String> {
        // Find the process
        match get_server_pid(process_name) {
            Some(pid) => {
                let mut sys = System::new_all();
                sys.refresh_all();

                if let Some(process) = sys.process(pid) {
                    // Try to kill the process
                    if process.kill() {
                        Ok(format!(
                            "Successfully sent kill signal to process '{}' (PID: {:?})",
                            process_name, pid
                        ))
                    } else {
                        Err(format!(
                            "Failed to kill process '{}' (PID: {:?})",
                            process_name, pid
                        ))
                    }
                } else {
                    Err(format!(
                        "Process '{}' with PID {:?} not found in system",
                        process_name, pid
                    ))
                }
            }
            None => Err(format!("Process '{}' not found", process_name)),
        }
    }
}
