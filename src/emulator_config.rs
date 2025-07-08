use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Represents a single emulator configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct Emulator {
    pub name: String,
    pub path: PathBuf,
    pub extensions: Vec<String>,
    #[serde(default)]
    pub core_path: Option<PathBuf>, // For RetroArch cores (optional, will be null for MAME-only setup)
    #[serde(default)]
    pub system_name: Option<String>, // For MAME console system short names (e.g., "genesis", "nes")
}

/// Represents the overall emulator configuration, containing a list of emulators.
#[derive(Debug, Serialize, Deserialize)]
pub struct EmulatorConfig {
    #[serde(rename = "emulators")] // Map JSON root array to a field named "emulators" for clarity
    pub emulators: Vec<Emulator>,
}

impl EmulatorConfig {
    /// Loads emulator configurations from a specified JSON file.
    ///
    /// The JSON file is expected to be an array of emulator objects.
    ///
    /// # Arguments
    /// * `path` - The path to the JSON configuration file.
    ///
    /// # Returns
    /// A `Result` containing an `EmulatorConfig` if successful, or an `io::Error` on failure.
    pub fn load(path: &Path) -> io::Result<Self> {
        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Emulator configuration file not found: {}", path.display()),
            ));
        }

        let contents = fs::read_to_string(path)?;
        let emulators: Vec<Emulator> = serde_json::from_str(&contents)
            .map_err(|e| io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse emulator config JSON: {}", e),
            ))?;

        Ok(EmulatorConfig { emulators })
    }

    /// (Optional) Saves the current emulator configurations to a JSON file.
    /// Useful if you implement configuration editing within the application.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let contents = serde_json::to_string_pretty(&self.emulators)
            .map_err(|e| io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize emulator config to JSON: {}", e),
            ))?;
        fs::write(path, contents)?;
        Ok(())
    }
}