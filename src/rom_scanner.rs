use std::path::{Path, PathBuf};
use std::io;
use walkdir::WalkDir;

/// Represents a found ROM file.
#[derive(Debug)]
pub struct Rom {
    pub path: PathBuf,
}

impl Rom {
    /// Gets the file extension of the ROM.
    pub fn get_extension(&self) -> Option<&str> {
        self.path.extension().and_then(|ext| ext.to_str())
    }
}

/// Scans a directory for ROM files based on provided extensions.
pub struct RomScanner<'a> {
    base_dir: &'a Path,
    supported_extensions: &'a [&'a str],
}

impl<'a> RomScanner<'a> {
    /// Creates a new `RomScanner`.
    ///
    /// # Arguments
    /// * `base_dir` - The directory to start scanning from.
    /// * `supported_extensions` - A slice of file extensions (e.g., `["nes", "snes"]`) to look for.
    pub fn new(base_dir: &'a Path, supported_extensions: &'a [&'a str]) -> Self {
        RomScanner {
            base_dir,
            supported_extensions,
        }
    }

    /// Scans the `base_dir` recursively for supported ROM files.
    ///
    /// # Returns
    /// A `Result` containing a `Vec<Rom>` if successful, or an `io::Error` on failure.
    pub fn scan_roms(&self) -> io::Result<Vec<Rom>> {
        let mut roms = Vec::new();

        // Check if the base directory exists.
        if !self.base_dir.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("ROMs directory not found: {}", self.base_dir.display()),
            ));
        }
        if !self.base_dir.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Path is not a directory: {}", self.base_dir.display()),
            ));
        }

        println!("🔍 Scanning for ROMs in: {}", self.base_dir.display());

        // Walk the directory recursively.
        for entry in WalkDir::new(self.base_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                // Detailed logging for each file encountered
                println!("  -- Checking file: {}", path.display());

                if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                    // Check if the file's extension is in our list of supported extensions.
                    if self.supported_extensions.iter().any(|&ext| ext.eq_ignore_ascii_case(extension)) {
                        println!("  -- Found supported ROM: {}", path.display()); // Log supported ROMs
                        roms.push(Rom { path: path.to_path_buf() });
                    } else {
                        println!("  -- Skipping file (unsupported extension: '{}'): {}", extension, path.display()); // Log skipped files
                    }
                } else {
                    println!("  -- Skipping file (no extension): {}", path.display()); // Log files without extensions
                }
            }
        }

        Ok(roms)
    }
}