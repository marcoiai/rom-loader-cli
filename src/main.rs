mod emulator_config;
mod rom_launcher;
mod rom_scanner;

use clap::Parser;
use emulator_config::{Emulator, EmulatorConfig};
use rom_scanner::{Rom, RomScanner};
use std::collections::HashMap;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

// Supported ROM extensions that the scanner will look for.
const SUPPORTED_ROM_EXTENSIONS: &[&str] = &["nes", "snes", "smc", "sfc", "gb", "gba", "n64", "ps1", "md", "gen", "bin", "zip", "7z"];

/// Command-line arguments for the ROM Loader.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the directory containing ROMs.
    #[arg(short, long, value_name = "DIR")]
    roms_dir: String,

    /// Path to the JSON configuration file for emulators.
    #[arg(short, long, value_name = "FILE", default_value = "emulators.json")]
    config_file: String,
}

fn main() -> io::Result<()> {
    // Parse command-line arguments.
    let args = Args::parse();

    println!("🚀 Starting ROM Loader...");

    // 1. Load Emulator Configuration
    let config_path = PathBuf::from(&args.config_file);
    let emulator_config = match EmulatorConfig::load(&config_path) {
        Ok(config) => {
            println!("✅ Loaded emulator configuration from: {}", config_path.display());
            config
        }
        Err(e) => {
            eprintln!("❌ Error loading emulator configuration from {}: {}", config_path.display(), e);
            eprintln!("Please ensure 'emulators.json' exists and is correctly formatted.");
            return Ok(());
        }
    };

    // Create a mapping from file extension to the preferred emulator.
    // This allows quick lookup of which emulator to use for a given ROM extension.
    let mut extension_to_emulator: HashMap<String, &Emulator> = HashMap::new();
    for emulator in &emulator_config.emulators {
        for ext in &emulator.extensions {
            // Prioritize the first emulator found for an extension.
            // In a more advanced setup, you might allow users to set preferences.
            extension_to_emulator.entry(ext.to_lowercase()).or_insert(emulator);
        }
    }

    // 2. Scan for ROMs
    let roms_dir_path = PathBuf::from(&args.roms_dir);
    let rom_scanner = RomScanner::new(&roms_dir_path, SUPPORTED_ROM_EXTENSIONS);

    let roms = match rom_scanner.scan_roms() {
        Ok(r) => {
            if r.is_empty() {
                println!("⚠️ No supported ROMs found in {}.", roms_dir_path.display());
                return Ok(());
            }
            println!("📚 Found {} ROMs in {}:", r.len(), roms_dir_path.display());
            r
        }
        Err(e) => {
            eprintln!("❌ Error scanning ROMs in {}: {}", roms_dir_path.display(), e);
            return Ok(());
        }
    };

    // Function to display the ROM list. This is now callable from multiple places.
    let display_rom_list = |roms: &[Rom], ext_to_emu: &HashMap<String, &Emulator>| {
        println!("\n--- Current ROMs List ---");
        for (i, rom) in roms.iter().enumerate() {
            let suggested_emulator_name = rom.path.extension()
                .and_then(|ext_os| ext_os.to_str())
                .and_then(|ext_str| ext_to_emu.get(&ext_str.to_lowercase()))
                .map_or("Unknown".to_string(), |e| e.name.clone());

            println!(
                "  {}. {} (Type: {}, Suggested Emulator: {})",
                i + 1,
                rom.path.file_name().unwrap_or_default().to_string_lossy(),
                rom.get_extension().unwrap_or("unknown"),
                suggested_emulator_name
            );
        }
        println!("-------------------------\n");
    };

    // Initial display of ROMs
    display_rom_list(&roms, &extension_to_emulator);

    // 3. User Selection and Launch
    loop {
        print!("🔢 Enter the number of the ROM to launch, 'l' to list games, or 'q' to quit: ");
        io::stdout().flush()?; // Ensure the prompt is displayed.

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.eq_ignore_ascii_case("q") {
            println!("👋 Exiting ROM Loader. Goodbye!");
            break;
        } else if input.eq_ignore_ascii_case("l") {
            display_rom_list(&roms, &extension_to_emulator);
        } else {
            match input.parse::<usize>() {
                Ok(num) if num > 0 && num <= roms.len() => {
                    let selected_rom = &roms[num - 1];
                    println!("You selected: {}", selected_rom.path.file_name().unwrap_or_default().to_string_lossy());

                    // Find the appropriate emulator for the selected ROM.
                    let rom_extension = selected_rom.get_extension().unwrap_or("").to_lowercase();
                    if let Some(emulator) = extension_to_emulator.get(&rom_extension) {
                        println!("Launching {} with {}...",
                            selected_rom.path.file_name().unwrap_or_default().to_string_lossy(),
                            emulator.name
                        );
                        // Pass emulator name, core path, AND system name for specific handling
                        if let Err(e) = rom_launcher::launch_rom(
                            &emulator.path,
                            &selected_rom.path,
                            &emulator.name,
                            emulator.core_path.as_ref(),
                            emulator.system_name.as_ref()
                        ) {
                            eprintln!("❌ Failed to launch emulator: {}", e);
                        } else {
                            println!("✅ Launch command sent.");
                        }
                    } else {
                        eprintln!("❌ No configured emulator found for '{}' files.", rom_extension);
                        eprintln!("Please add an entry to your 'emulators.json' for this ROM type.");
                    }
                }
                _ => {
                    println!("🚫 Invalid selection. Please enter a valid number, 'l', or 'q'.");
                }
            }
        }
    }

    Ok(())
}