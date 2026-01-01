use sysinfo::{Pid, System};

pub fn get_server_pid(process_name: &str) -> Option<Pid> {
    let mut sys = System::new_all();
    sys.refresh_all();

    for (pid, process) in sys.processes() {
        if process.name()
          .to_ascii_lowercase()
          .contains(
            &process_name.to_ascii_lowercase()
          ) {
            return Some(*pid);
        }
    }
    None
}
