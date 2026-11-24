//! m913ctl: configure the Redragon M913 (Impact Elite) on Linux.
//!
//! The protocol was captured with usbmon from Redragon's Windows tool: every request is feature
//! report 0x08 on interface 1, answered by input report 0x09 echoing it.
//!   08 08 00 HI LO N            read N (<=10) bytes   -> 09 08 00 HI LO N <data>
//!   08 07 00 HI LO N <data>     write N (<=10) bytes  -> echoed
//! Byte 15 is a checksum. The vendor tool's Apply is only writes; there is no commit command.

mod config;
mod device;
mod dump;
mod protocol;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use device::Mouse;

/// Configure the Redragon M913 (Impact Elite): buttons, macros, DPI, lighting, polling rate.
#[derive(Parser)]
#[command(version, after_help = "See the README for the config format.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print the mouse's current settings in the config format
    Dump,
    /// Write a config to the mouse; only the sections present in it are written
    Apply {
        /// Defaults to $XDG_CONFIG_HOME/m913ctl/config.yaml
        file: Option<PathBuf>,
    },
    /// Validate a config without touching the mouse
    Check {
        /// Defaults to $XDG_CONFIG_HOME/m913ctl/config.yaml
        file: Option<PathBuf>,
    },
    /// Save the mouse's settings memory to a file
    Backup { file: PathBuf },
    /// Write a backup back to the mouse
    Restore { file: PathBuf },
}

fn default_config() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config"));
    base.join("m913ctl/config.yaml")
}

fn plan(path: &Path) -> Result<Vec<config::Write>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    config::plan(&text).map_err(|e| format!("{}: {e}", path.display()))
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Dump => print!("{}", dump::dump(&mut Mouse::open()?)?),
        Command::Check { file } => {
            let path = file.unwrap_or_else(default_config);
            let writes = plan(&path)?;
            println!("{}: ok ({} writes)", path.display(), writes.len());
        }
        Command::Apply { file } => {
            let writes = plan(&file.unwrap_or_else(default_config))?;
            let mut mouse = Mouse::open()?;
            for w in writes {
                mouse.write(w.addr, &w.data)?;
                if !w.what.is_empty() {
                    println!("{}", w.what);
                }
            }
        }
        Command::Backup { file } => {
            let mem = Mouse::open()?.read(0, protocol::BACKUP_END)?;
            std::fs::write(&file, mem).map_err(|e| format!("{}: {e}", file.display()))?;
        }
        Command::Restore { file } => {
            let mem = std::fs::read(&file).map_err(|e| format!("{}: {e}", file.display()))?;
            if mem.len() != protocol::BACKUP_END {
                return Err(format!("{} is not an m913ctl backup", file.display()));
            }
            let mut mouse = Mouse::open()?;
            for (addr, len) in protocol::BACKUP_RANGES {
                mouse.write(addr, &mem[addr as usize..addr as usize + len])?;
            }
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("m913ctl: {e}");
            ExitCode::FAILURE
        }
    }
}
