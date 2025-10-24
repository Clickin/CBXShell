//! Debug logging utility for diagnosing thumbnail extraction issues
//!
//! Provides file-based logging that persists across DLL loads/unloads
//! to help diagnose why Windows Explorer may not be showing thumbnails.

use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

/// Global debug log file path
const DEBUG_LOG_PATH: &str = "G:\\CBXTest\\cbxshell_debug.log";

/// Global mutex to serialize log writes
static LOG_MUTEX: Mutex<()> = Mutex::new(());

/// Log a debug message to file with timestamp
///
/// This function is safe to call from any thread and will serialize writes.
/// Errors are silently ignored to prevent logging from breaking functionality.
pub fn debug_log(msg: &str) {
    let _guard = LOG_MUTEX.lock().unwrap();

    let _ = OpenOptions::new()
        .create(true)
        .append(true)
        .open(DEBUG_LOG_PATH)
        .and_then(|mut f| {
            use std::time::SystemTime;

            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            writeln!(f, "[{}] {}", timestamp, msg)
        });
}

/// Log method entry with parameters
#[macro_export]
macro_rules! log_entry {
    ($method:expr) => {
        $crate::utils::debug_log::debug_log(&format!("[ENTRY] {}", $method));
    };
    ($method:expr, $($arg:tt)*) => {
        $crate::utils::debug_log::debug_log(&format!("[ENTRY] {} - {}", $method, format!($($arg)*)));
    };
}

/// Log method success with result
#[macro_export]
macro_rules! log_success {
    ($method:expr) => {
        $crate::utils::debug_log::debug_log(&format!("[SUCCESS] {}", $method));
    };
    ($method:expr, $($arg:tt)*) => {
        $crate::utils::debug_log::debug_log(&format!("[SUCCESS] {} - {}", $method, format!($($arg)*)));
    };
}

/// Log method failure with error
#[macro_export]
macro_rules! log_error {
    ($method:expr, $error:expr) => {
        $crate::utils::debug_log::debug_log(&format!("[ERROR] {} - {}", $method, $error));
    };
}

/// Clear the debug log file (useful for testing)
pub fn clear_debug_log() {
    let _ = std::fs::remove_file(DEBUG_LOG_PATH);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_log_basic() {
        clear_debug_log();
        debug_log("Test message");

        let contents = std::fs::read_to_string(DEBUG_LOG_PATH).unwrap();
        assert!(contents.contains("Test message"));
    }

    #[test]
    fn test_debug_log_concurrent() {
        use std::thread;

        clear_debug_log();

        // Small delay to ensure file is deleted
        std::thread::sleep(std::time::Duration::from_millis(100));

        let handles: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    debug_log(&format!("Thread {} message", i));
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let contents = std::fs::read_to_string(DEBUG_LOG_PATH).unwrap();

        // Count only lines containing "Thread" and "message" from this test
        // Other tests may write to the log file concurrently
        let matching_lines = contents.lines()
            .filter(|line| line.contains("Thread") && line.contains("message"))
            .count();

        // Verify we have exactly 10 messages from our threads
        assert_eq!(matching_lines, 10,
            "Expected 10 thread messages, found {} (total lines: {})",
            matching_lines, contents.lines().count());
    }
}
