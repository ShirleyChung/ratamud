use std::sync::Mutex;
use once_cell::sync::Lazy;

/// Output zones for different types of game messages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputZone {
    Main,      // Main game output (room descriptions, actions, etc.)
    Log,       // System logs and debug messages
    Status,    // Status bar (time, health, etc.)
    Side,      // Side panel (inventory, map, etc.)
}

impl OutputZone {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutputZone::Main => "MAIN",
            OutputZone::Log => "LOG",
            OutputZone::Status => "STATUS",
            OutputZone::Side => "SIDE",
        }
    }
}

/// Callback function type for output
pub type OutputCallback = Box<dyn Fn(OutputZone, &str) + Send + Sync>;

/// Global output callback storage
static OUTPUT_CALLBACK: Lazy<Mutex<Option<OutputCallback>>> = Lazy::new(|| Mutex::new(None));

/// Core output manager for non-UI mode
pub struct CoreOutputManager {
    messages: Vec<String>,
    log_messages: Vec<String>,
    status: String,
    side_content: String,
}

impl Default for CoreOutputManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CoreOutputManager {
    pub fn new() -> Self {
        CoreOutputManager {
            messages: Vec::new(),
            log_messages: Vec::new(),
            status: String::new(),
            side_content: String::new(),
        }
    }

    /// Add a message to the main output
    pub fn add_message(&mut self, msg: String) {
        self.messages.push(msg.clone());
        trigger_output(OutputZone::Main, &msg);
    }

    /// Add a log message
    pub fn add_log(&mut self, msg: String) {
        self.log_messages.push(msg.clone());
        trigger_output(OutputZone::Log, &msg);
    }

    /// Set status message
    pub fn set_status(&mut self, msg: String) {
        self.status = msg.clone();
        trigger_output(OutputZone::Status, &msg);
    }

    /// Set side panel content
    pub fn set_side_content(&mut self, content: String) {
        self.side_content = content.clone();
        trigger_output(OutputZone::Side, &content);
    }

    /// Get all messages
    #[allow(dead_code)]
    pub fn get_messages(&self) -> &[String] {
        &self.messages
    }

    /// Get all log messages
    #[allow(dead_code)]
    pub fn get_logs(&self) -> &[String] {
        &self.log_messages
    }

    /// Get status
    #[allow(dead_code)]
    pub fn get_status(&self) -> &str {
        &self.status
    }

    /// Get side content
    #[allow(dead_code)]
    pub fn get_side_content(&self) -> &str {
        &self.side_content
    }

    /// Clear main messages
    #[allow(dead_code)]
    pub fn clear_messages(&mut self) {
        self.messages.clear();
    }

    /// Clear log messages
    #[allow(dead_code)]
    pub fn clear_logs(&mut self) {
        self.log_messages.clear();
    }
}

/// Register a global output callback
pub fn register_output_callback<F>(callback: F)
where
    F: Fn(OutputZone, &str) + Send + Sync + 'static,
{
    let mut cb = OUTPUT_CALLBACK.lock().unwrap();
    *cb = Some(Box::new(callback));
}

/// Trigger the output callback
pub fn trigger_output(zone: OutputZone, content: &str) {
    let cb = OUTPUT_CALLBACK.lock().unwrap();
    if let Some(callback) = cb.as_ref() {
        callback(zone, content);
    }
}

/// Clear the output callback
pub fn clear_output_callback() {
    let mut cb = OUTPUT_CALLBACK.lock().unwrap();
    *cb = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_manager() {
        let mut manager = CoreOutputManager::new();
        manager.add_message("Hello".to_string());
        assert_eq!(manager.get_messages().len(), 1);
        assert_eq!(manager.get_messages()[0], "Hello");
    }

    #[test]
    fn test_output_zones() {
        assert_eq!(OutputZone::Main.as_str(), "MAIN");
        assert_eq!(OutputZone::Log.as_str(), "LOG");
        assert_eq!(OutputZone::Status.as_str(), "STATUS");
        assert_eq!(OutputZone::Side.as_str(), "SIDE");
    }
}
