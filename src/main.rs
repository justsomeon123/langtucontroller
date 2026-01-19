use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{Shell, generate};
use std::fs;
use std::io::{self, ErrorKind, Write};
use std::os::unix::fs::FileTypeExt;
use std::path::PathBuf;

enum ConnectionType {
    Usb,
    Wifi,
    Unknown,
}

fn get_hidraws() -> io::Result<Vec<PathBuf>> {
    let mut hidraws = Vec::new();
    for entry in fs::read_dir("/dev")? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with("hidraw") && entry.file_type()?.is_char_device() {
            hidraws.push(entry.path());
        }
    }
    Ok(hidraws)
}

fn determine_connection(paths: &[PathBuf]) -> ConnectionType {
    match paths.len() {
        6 => ConnectionType::Usb,
        7 => ConnectionType::Wifi,
        _ => ConnectionType::Unknown,
    }
}

fn get_keyboard_hid() -> io::Result<Option<PathBuf>> {
    let hidraws = get_hidraws()?;

    match determine_connection(&hidraws) {
        ConnectionType::Wifi => {
            for hidraw in hidraws {
                let syspath = PathBuf::from(format!(
                    "/sys/class/hidraw/{}/device",
                    hidraw
                        .file_name()
                        .ok_or(ErrorKind::NotFound)?
                        .to_string_lossy()
                ));

                // Use canonicalize to resolve symlinks in /sys/class
                if let Ok(devpath) = fs::canonicalize(syspath) {
                    let subclass = if let Some(path) =
                        devpath.parent().map(|p| p.join("bInterfaceSubClass"))
                    {
                        fs::read(path).ok().and_then(|v| String::from_utf8(v).ok())
                    } else {
                        None
                    };

                    let product = if let Some(path) = devpath
                        .parent()
                        .and_then(|p| p.parent())
                        .map(|p| p.join("idProduct"))
                    {
                        fs::read(path).ok().and_then(|v| String::from_utf8(v).ok())
                    } else {
                        None
                    };

                    let vendor = if let Some(path) = devpath
                        .parent()
                        .and_then(|p| p.parent())
                        .map(|p| p.join("idVendor"))
                    {
                        fs::read(path).ok().and_then(|v| String::from_utf8(v).ok())
                    } else {
                        None
                    };

                    if let (Some(subclass), Some(vendor), Some(product)) =
                        (subclass, vendor, product)
                        && (subclass.trim() == "00" || subclass.trim() == "01")
                        && (product.trim() == "7fff" || product.trim() == "484a")
                        && vendor.trim() == "1a2c"
                    {
                        return Ok(Some(hidraw));
                    }
                }
            }
        }
        ConnectionType::Usb => {
            eprintln!("No USB support yet!")
        }
        ConnectionType::Unknown => eprintln!("Unknown connection type"),
    };

    Ok(None)
}

fn write_to_keyboard(
    hidpath: PathBuf,
    mode: u8,
    val1: u8,
    val2: u8,
    val3: u8,
    brightness: u8,
) -> io::Result<()> {
    let mut buf: [u8; 64] = [0; 64];
    buf[..5].copy_from_slice(&[0xbb, 0xaa, 0x99, 0x88, 0xaa]);
    buf[5] = mode;
    buf[6] = 0x00;
    buf[7] = val1;
    buf[8] = val2;
    buf[9] = brightness;
    buf[10] = val3;

    let mut hid = fs::File::options().write(true).open(hidpath)?;
    hid.write_all(&buf)?;
    Ok(())
}

#[derive(Parser)]
#[command(name = "langtuctl")]
#[command(version = "1.0")]
#[command(about = "Allows for control of L98 keyboard")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Rainbow flowing effect
    Rainbow {
        /// Direction of animation (even right, odd left)
        #[arg(value_hint = clap::ValueHint::Other)]
        direction: u8,

        /// Speed of rainbow cycling (0 is slowest)
        #[arg(value_hint = clap::ValueHint::Other)]
        speed: u8,

        /// Brightness of keyboard (0-4)
        #[arg(value_hint = clap::ValueHint::Other, value_parser = clap::value_parser!(u8).range(0..=4))]
        brightness: u8,
    },

    /// Static color mode
    Color {
        /// Red channel intensity (0-255)
        #[arg(value_hint = clap::ValueHint::Other)]
        red: u8,

        /// Green channel intensity (0-255)
        #[arg(value_hint = clap::ValueHint::Other)]
        green: u8,

        /// Blue channel intensity (0-255)
        #[arg(value_hint = clap::ValueHint::Other)]
        blue: u8,

        /// Brightness of keyboard (0-4)
        #[arg(value_hint = clap::ValueHint::Other, value_parser = clap::value_parser!(u8).range(0..=4))]
        brightness: u8,
    },

    /// Generate shell completion scripts
    Completion {
        #[arg(value_enum)]
        shell: Shell,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Commands::Completion { shell } = cli.command {
        let mut cmd = Cli::command();
        let name = cmd.get_name().to_string();
        generate(shell, &mut cmd, name, &mut io::stdout());
        return;
    }

    let hidpath = match get_keyboard_hid() {
        Ok(Some(path)) => path,
        Ok(None) => {
            eprintln!("Keyboard not found (Check /dev/hidraw* permissions or connection)");
            return;
        }
        Err(e) => {
            eprintln!("Error scanning for device: {:?}", e);
            return;
        }
    };

    let (mode, val1, val2, val3, brightness) = match cli.command {
        Commands::Rainbow {
            direction,
            speed,
            brightness,
        } => {
            // Mapping: val1=direction, val2=speed, val3=0 (unused)
            (0x00, direction, speed, 0, brightness)
        }
        Commands::Color {
            red,
            green,
            blue,
            brightness,
        } => {
            // Mapping: val1=red, val2=green, val3=blue
            (0x03, red, green, blue, brightness)
        }
        Commands::Completion { .. } => unreachable!(),
    };

    match write_to_keyboard(hidpath, mode, val1, val2, val3, brightness) {
        Ok(()) => println!("Successfully updated keyboard settings."),
        Err(e) => eprintln!("Failed to write to keyboard: {:?}", e),
    };
}
