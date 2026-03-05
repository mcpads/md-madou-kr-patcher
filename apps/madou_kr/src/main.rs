use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};
use std::process;

use madou_kr::align;
use madou_kr::bps;
use madou_kr::build;
use madou_kr::check;
use madou_kr::extract;
use madou_kr::ips;
use madou_kr::rom;

#[derive(Parser)]
#[command(name = "madou-kr")]
#[command(about = "Madou Monogatari I Korean patch tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a BPS patch from source and target ROMs
    Create {
        /// Source ROM (English v1.1)
        #[arg(long)]
        source: PathBuf,
        /// Target ROM (Korean patched)
        #[arg(long)]
        target: PathBuf,
        /// Output BPS patch file
        #[arg(long)]
        output: PathBuf,
    },
    /// Apply a BPS patch to a source ROM
    Apply {
        /// Source ROM (English v1.1)
        #[arg(long)]
        rom: PathBuf,
        /// BPS patch file
        #[arg(long)]
        patch: PathBuf,
        /// Output patched ROM
        #[arg(long)]
        output: PathBuf,
    },
    /// Apply an IPS patch to a source ROM
    ApplyIps {
        /// Source ROM (JP)
        #[arg(long)]
        rom: PathBuf,
        /// IPS patch file
        #[arg(long)]
        patch: PathBuf,
        /// Output patched ROM
        #[arg(long)]
        output: PathBuf,
    },
    /// Build KR ROM from EN ROM + assets directory, then optionally generate BPS patch
    Build {
        /// ROM path (EN v1.1, or JP if --ips is provided)
        #[arg(long)]
        rom: PathBuf,
        /// IPS patch to apply to JP ROM first (produces EN ROM in-memory)
        #[arg(long)]
        ips: Option<PathBuf>,
        /// Assets directory (translation/, charmap.json, neodgm.ttf)
        #[arg(long)]
        assets: PathBuf,
        /// Output KR ROM path
        #[arg(long)]
        output: PathBuf,
        /// Also generate BPS patch file
        #[arg(long)]
        bps: Option<PathBuf>,
    },
    /// Check control code integrity (EN vs KR)
    CheckCtrl {
        /// Assets directory
        #[arg(long)]
        assets: PathBuf,
    },
    /// Check text overflow (pixel width)
    CheckOverflow {
        /// ROM path (EN v1.1, or JP if --ips is provided)
        #[arg(long)]
        rom: PathBuf,
        /// IPS patch to apply to JP ROM first (produces EN ROM in-memory)
        #[arg(long)]
        ips: Option<PathBuf>,
        /// Assets directory
        #[arg(long)]
        assets: PathBuf,
    },
    /// Generate derived assets (charmap.json, en_reference.json, text_en.json) from EN ROM
    Init {
        /// ROM path (EN v1.1, or JP if --ips is provided)
        #[arg(long)]
        rom: PathBuf,
        /// IPS patch to apply to JP ROM first (produces EN ROM in-memory)
        #[arg(long)]
        ips: Option<PathBuf>,
        /// Assets output directory
        #[arg(long)]
        assets: PathBuf,
    },
    /// Generate JP-EN-KR complete text alignment (chunked JSON files)
    Align {
        /// JP ROM path
        #[arg(long)]
        jp_rom: PathBuf,
        /// EN ROM path (or JP ROM if --ips provided)
        #[arg(long)]
        en_rom: PathBuf,
        /// IPS patch (optional, if en_rom is actually JP ROM)
        #[arg(long)]
        ips: Option<PathBuf>,
        /// Assets directory
        #[arg(long)]
        assets: PathBuf,
        /// Output directory for chunked JSON files
        #[arg(long)]
        output: PathBuf,
        /// Entries per JSON file
        #[arg(long, default_value = "32")]
        chunk_size: usize,
    },
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Create { source, target, output } => {
            cmd_create(&source, &target, &output)
        }
        Commands::Apply { rom, patch, output } => {
            cmd_apply(&rom, &patch, &output)
        }
        Commands::ApplyIps { rom, patch, output } => {
            cmd_apply_ips(&rom, &patch, &output)
        }
        Commands::Build { rom, ips: ips_patch, assets, output, bps } => {
            cmd_build(&rom, ips_patch.as_deref(), &assets, &output, bps.as_deref())
        }
        Commands::CheckCtrl { assets } => {
            check::ctrl_codes::run(&assets).map_err(|e| e.into())
        }
        Commands::CheckOverflow { rom, ips: ips_patch, assets } => {
            cmd_check_overflow(&rom, ips_patch.as_deref(), &assets)
        }
        Commands::Init { rom, ips: ips_patch, assets } => {
            cmd_init(&rom, ips_patch.as_deref(), &assets)
        }
        Commands::Align { jp_rom, en_rom, ips: ips_patch, assets, output, chunk_size } => {
            cmd_align(&jp_rom, &en_rom, ips_patch.as_deref(), &assets, &output, chunk_size)
        }
    };
    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}

fn cmd_create(source: &PathBuf, target: &PathBuf, output: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let source_data = std::fs::read(source)?;
    let target_data = std::fs::read(target)?;
    rom::validate_rom(&source_data, "source")?;
    let patch = bps::create(&source_data, &target_data)?;
    std::fs::write(output, &patch)?;
    println!("Patch created: {} ({} bytes)", output.display(), patch.len());
    Ok(())
}

fn cmd_apply(rom_path: &PathBuf, patch_path: &PathBuf, output: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let source_data = std::fs::read(rom_path)?;
    let patch_data = std::fs::read(patch_path)?;
    rom::validate_rom(&source_data, "ROM")?;
    let target_data = bps::apply(&source_data, &patch_data)?;
    std::fs::write(output, &target_data)?;
    println!("Patch applied: {} ({} bytes)", output.display(), target_data.len());
    Ok(())
}

fn cmd_apply_ips(
    rom_path: &PathBuf,
    patch_path: &PathBuf,
    output: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let source_data = std::fs::read(rom_path)?;
    let patch_data = std::fs::read(patch_path)?;
    let target_data = ips::apply(&source_data, &patch_data)?;
    rom::validate_rom(&target_data, "patched ROM")?;
    std::fs::write(output, &target_data)?;
    println!("IPS patch applied: {} ({} bytes)", output.display(), target_data.len());
    Ok(())
}

/// Load EN ROM, optionally by applying IPS patch to JP ROM first.
fn load_en_rom(rom_path: &Path, ips_path: Option<&Path>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let rom_data = std::fs::read(rom_path)?;
    match ips_path {
        Some(ips) => {
            println!("Applying IPS patch: {}", ips.display());
            let patch_data = std::fs::read(ips)?;
            let en_rom = ips::apply(&rom_data, &patch_data)?;
            rom::validate_rom(&en_rom, "EN ROM (after IPS)")?;
            println!("  JP ROM {} → EN ROM {} bytes", rom_data.len(), en_rom.len());
            Ok(en_rom)
        }
        None => {
            rom::validate_rom(&rom_data, "EN ROM")?;
            Ok(rom_data)
        }
    }
}

fn cmd_build(
    rom_path: &Path,
    ips_path: Option<&Path>,
    assets_dir: &Path,
    output_path: &Path,
    bps_path: Option<&Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    // If --ips provided, write EN ROM to temp file for BuildConfig
    let en_rom_data = load_en_rom(rom_path, ips_path)?;
    let en_rom_tmp;
    let en_rom_path: &Path = if ips_path.is_some() {
        en_rom_tmp = std::env::temp_dir().join("madou_kr_en_rom.md");
        std::fs::write(&en_rom_tmp, &en_rom_data)?;
        &en_rom_tmp
    } else {
        rom_path
    };

    let config = build::BuildConfig {
        en_rom_path,
        assets_dir,
        output_path,
    };

    let kr_rom = build::build_kr_rom(&config)?;

    // Save KR ROM
    std::fs::write(output_path, &kr_rom)?;
    println!("KR ROM saved: {} ({} bytes)", output_path.display(), kr_rom.len());

    // Optionally generate BPS patch (always against EN ROM, not JP)
    if let Some(bps_out) = bps_path {
        let patch = bps::create(&en_rom_data, &kr_rom)?;
        std::fs::write(bps_out, &patch)?;
        println!("BPS patch saved: {} ({} bytes)", bps_out.display(), patch.len());
    }

    // Clean up temp file
    if ips_path.is_some() {
        let _ = std::fs::remove_file(en_rom_path);
    }

    Ok(())
}

fn cmd_check_overflow(
    rom_path: &Path,
    ips_path: Option<&Path>,
    assets_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let en_rom_data = load_en_rom(rom_path, ips_path)?;
    let en_rom_tmp = std::env::temp_dir().join("madou_kr_overflow_en.md");
    std::fs::write(&en_rom_tmp, &en_rom_data)?;
    let result = check::overflow::run(&en_rom_tmp, assets_dir).map_err(|e| -> Box<dyn std::error::Error> { e.into() });
    let _ = std::fs::remove_file(&en_rom_tmp);
    result
}

fn cmd_align(
    jp_rom_path: &Path,
    en_rom_path: &Path,
    ips_path: Option<&Path>,
    assets_dir: &Path,
    output_dir: &Path,
    chunk_size: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let jp_rom_data = std::fs::read(jp_rom_path)?;
    let en_rom_data = load_en_rom(en_rom_path, ips_path)?;
    align::run(&jp_rom_data, &en_rom_data, assets_dir, output_dir, chunk_size)
}

fn cmd_init(
    rom_path: &Path,
    ips_path: Option<&Path>,
    assets_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let en_rom_data = load_en_rom(rom_path, ips_path)?;
    let en_rom_tmp;
    let en_rom_path: &Path = if ips_path.is_some() {
        en_rom_tmp = std::env::temp_dir().join("madou_kr_en_rom.md");
        std::fs::write(&en_rom_tmp, &en_rom_data)?;
        &en_rom_tmp
    } else {
        rom_path
    };

    let result = extract::run(en_rom_path, assets_dir);

    if ips_path.is_some() {
        let _ = std::fs::remove_file(en_rom_path);
    }

    result
}
