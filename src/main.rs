#![windows_subsystem = "windows"]

extern crate registry;
extern crate winreg;

use auto_dark_mode::{install, uninstall, update_theme, watch_registry, Args};

use clap::Parser;
use registry::{Hive, Security};
use std::io;
use winapi::um::wincon::{AttachConsole, ATTACH_PARENT_PROCESS};
use winreg::{enums::*, RegKey};

const HKCU_THEMES_PERSONALIZE: &str =
    "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize";
const HKCU_BLUELIGHT_STATE: &str =
    "Software\\Microsoft\\Windows\\CurrentVersion\\CloudStore\\Store\\DefaultAccount\\Current\\default$windows.data.bluelightreduction.bluelightreductionstate\\windows.data.bluelightreduction.bluelightreductionstate";

fn main() -> io::Result<()> {
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }

    let args = Args::parse();

    if args.app_only && args.system_only {
        panic!("Cannot set app only and system only at the same time.");
    }

    if args.install {
        install(&args.install_dir, &args)?;
        println!("Done installing auto_dark_mode");
        return Ok(());
    }

    if args.uninstall {
        uninstall()?;
        println!("Done uninstalling auto_dark_mode");
        return Ok(());
    }

    println!("Starting auto_dark_mode by zac");
    println!("https://github.com/zaccnz/auto_dark_mode/");

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let personalize = hkcu.open_subkey_with_flags(HKCU_THEMES_PERSONALIZE, KEY_WRITE)?;

    let blue_light_registry = Hive::CurrentUser
        .open(HKCU_BLUELIGHT_STATE, Security::Read)
        .unwrap();

    update_theme(&blue_light_registry, &personalize, &args)?;

    let receiver = watch_registry(hkcu, HKCU_BLUELIGHT_STATE)?;

    println!("Listening to changes...");

    loop {
        let res = receiver.recv().unwrap();
        println!("{:?}.", res);

        update_theme(&blue_light_registry, &personalize, &args)?;
    }
}
