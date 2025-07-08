use std::path::{Path, PathBuf};
use std::process::Command;
use std::io;

/// Launches an emulator with a specified ROM file.
///
/// This function attempts to execute the emulator program, passing the ROM path as an argument.
/// Special handling is included for MAME and RetroArch, which typically require specific arguments.
///
/// # Arguments
/// * `emulator_path` - The path to the emulator executable.
/// * `rom_path` - The path to the ROM file to be launched.
/// * `emulator_name` - The name of the emulator, used to identify MAME or RetroArch.
/// * `core_path` - An optional path to the RetroArch core, if applicable.
/// * `system_name` - An optional MAME system short name (e.g., "genesis", "nes") for console ROMs.
///
/// # Returns
/// A `Result` indicating success or an `io::Error` if the command fails to execute.
pub fn launch_rom(
    emulator_path: &Path,
    rom_path: &Path,
    emulator_name: &str,
    core_path: Option<&PathBuf>,
    system_name: Option<&String>, // New argument
) -> io::Result<()> {
    if !emulator_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Emulator executable not found: {}", emulator_path.display()),
        ));
    }

    if !emulator_path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Emulator path is not an executable file: {}", emulator_path.display()),
            ));
    }

    let mut command = Command::new(emulator_path);
    let emulator_name_lower = emulator_name.to_lowercase();

    if emulator_name_lower.contains("mame") {
        if let Some(sys_name) = system_name {
            // MAME for consoles: <mame_exe> <system_name> -cart <full_rom_path>
            // MAME expects the ROM path for -cart, not just the file stem.
            command.arg(sys_name).arg("-cart").arg(rom_path);
            println!("  (MAME Console Command: {} {} -cart \"{}\")",
                     emulator_path.display(),
                     sys_name,
                     rom_path.display()
            );
        } else {
            // MAME for arcade: <mame_exe> -rompath <rom_dir> <rom_short_name>
            if let Some(parent_dir) = rom_path.parent() {
                command.arg("-rompath").arg(parent_dir);
            } else {
                eprintln!("⚠️ Warning: Could not determine ROM parent directory for MAME arcade. Launch might fail.");
            }

            if let Some(rom_file_name) = rom_path.file_stem().and_then(|s| s.to_str()) {
                command.arg(rom_file_name);
                println!("  (MAME Arcade Command: {} -rompath \"{}\" \"{}\")",
                         emulator_path.display(),
                         rom_path.parent().unwrap_or_else(|| Path::new("")).display(),
                         rom_file_name
                );
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Could not determine ROM file stem for MAME arcade: {}", rom_path.display()),
                ));
            }
        }
    } else if emulator_name_lower.contains("retroarch") {
        // RetroArch often needs a core specified with -L
        if let Some(core) = core_path {
            if !core.exists() || !core.is_file() {
                eprintln!("❌ RetroArch core not found or not a file: {}. Launch might fail.", core.display());
            }
            command.arg("-L").arg(core); // Specify the core
            command.arg(rom_path);       // Then the ROM path
            println!("  (RetroArch Command: {} -L \"{}\" \"{}\")",
                     emulator_path.display(),
                     core.display(),
                     rom_path.display()
            );
        } else {
            // Fallback for RetroArch if no core path is provided in config
            command.arg(rom_path);
            println!("  (RetroArch Command (no core specified): {} \"{}\")",
                     emulator_path.display(),
                     rom_path.display()
            );
            eprintln!("⚠️ Warning: RetroArch may require a core path (-L argument). Please add 'core_path' to your emulators.json entry for RetroArch.");
        }
    } else {
        // Generic handling for other emulators: just pass the ROM path
        command.arg(rom_path);
        println!("  (Generic Command: {} \"{}\")", emulator_path.display(), rom_path.display());
    }

    let output = command.spawn()? // `spawn` starts the process and returns immediately.
        .wait_with_output()?; // `wait_with_output` waits for the process to finish.

    // You might want to inspect `output.status`, `output.stdout`, `output.stderr`
    // for more detailed error handling or logging.
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Emulator process exited with non-zero status: {:?}", output.status);
        if !stderr.is_empty() {
            eprintln!("Emulator stderr: {}", stderr);
        }
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Emulator process failed",
        ));
    }

    Ok(())
}