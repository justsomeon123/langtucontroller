use clap::Parser;
use clap::ValueEnum;
use std::fs;
use std::io;
use std::io::ErrorKind;
use std::io::Write;
use std::os::unix::fs::FileTypeExt;
use std::path::PathBuf;

//Supplementary methods
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

fn get_keyboard_hid() -> io::Result<Option<PathBuf>> {
    for hidraw in get_hidraws()? {
        let syspath = PathBuf::from(format!(
            "/sys/class/hidraw/{}/device",
            hidraw
                .file_name()
                .ok_or(ErrorKind::NotFound)?
                .to_string_lossy()
        ));

        let devpath = fs::canonicalize(syspath)?;

        let subclass = if let Some(path) = devpath.parent().map(|p| p.join("bInterfaceSubClass")) {
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

        if let (Some(subclass), Some(vendor), Some(product)) = (subclass, vendor, product) {
            if subclass.trim() == "00" && product.trim() == "7fff" && vendor.trim() == "1a2c" {
                return Ok(Some(hidraw));
            }
        } else {
            eprintln!("Malformed/unreadable hidraw, {:?}", hidraw);
        }
    }
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

//All the real CLI stuff.
#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Mode {
    //Rainbow mode (direction, speed, n/a, brightness)
    Rainbow = 0x00,
    //Color mode (r,g,b,brightness)
    Color = 0x03,
}

#[derive(Parser)]
#[command(name = "Langtu L98 Controller")]
#[command(version = "1.0")]
#[command(about = "Allows for control of L98 keyboard\nDefault rainbow:(rainbow,0,2,0,4)")]
struct CLIInterface {
    #[arg(value_enum)]
    mode: Mode,

    #[arg(value_parser = clap::value_parser!(u8).range(0..))]
    val1: u8,

    #[arg(value_parser = clap::value_parser!(u8).range(0..))]
    val2: u8,

    #[arg(value_parser = clap::value_parser!(u8).range(0..))]
    val3: u8,

    #[arg(value_parser = clap::value_parser!(u8).range(0..=4))]
    brightness: u8,
}

fn main() {
    let hidpath;
    let cli = CLIInterface::parse();

    match get_keyboard_hid() {
        Ok(path) => {
            if let Some(path) = path {
                hidpath = path;
            } else {
                eprintln!("Failed to open HID device's path (does not exist)");
                return;
            }
        }
        Err(e) => {
            eprintln!("Errored out:\n {:?}", e);
            return;
        }
    }

    match write_to_keyboard(
        hidpath,
        cli.mode as u8,
        cli.val1,
        cli.val2,
        cli.val3,
        cli.brightness,
    ) {
        Ok(()) => println!("Successfully wrote to keyboard."),
        Err(e) => eprintln!("Failed to write to keyboard:\n {:?}", e),
    };
}
