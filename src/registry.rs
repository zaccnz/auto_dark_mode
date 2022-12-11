extern crate winapi;

use reg_watcher::{filter, WatchResponse, Watcher};
use registry::Data;
use std::ffi::OsStr;
use std::os::windows::prelude::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use std::{env, fs, io, iter, mem, ptr, result};
use sysinfo::{ProcessExt, System, SystemExt};
use winapi::shared::minwindef::DWORD;
use winapi::um::processthreadsapi::{PROCESS_INFORMATION, STARTUPINFOW};
use winapi::um::winbase::CREATE_NO_WINDOW;
use winreg::enums::{HKEY_CURRENT_USER, KEY_WRITE};
use winreg::RegKey;

use crate::Args;

const HKCU_STARTUP: &str = "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run";
const STARTUP_KEY: &str = "AutoDarkModeRs";
const INSTALL_DIR: &str = "auto_dark_mode";
const EXE_NAME: &str = "auto_dark_mode.exe";

pub fn update_theme(
    blue_light_registry: &registry::RegKey,
    personalize_write: &RegKey,
    args: &Args,
) -> io::Result<()> {
    println!("Reading night light state...");

    let blue_light_data: Vec<u8> = match blue_light_registry.value("Data") {
        Ok(Data::Binary(value)) => value,
        _ => panic!("Couldn't read binary data for blue light filter"),
    };

    let dark_theme =
        blue_light_data.len() > 24 && blue_light_data[23] == 0x10 && blue_light_data[24] == 0x00;

    println!(
        "Night light is {}.  Updating theme.",
        if dark_theme { "enabled" } else { "disabled" }
    );

    let value = if dark_theme { 0u32 } else { 1u32 };

    if !args.system_only {
        personalize_write.set_value("AppsUseLightTheme", &value)?;
    }
    if !args.app_only {
        personalize_write.set_value("SystemUsesLightTheme", &value)?;
    }

    Ok(())
}

pub fn watch_registry(predef: RegKey, path: &str) -> io::Result<Receiver<WatchResponse>> {
    let winreg = predef.open_subkey(path)?;

    let mut watcher = Watcher::new(
        winreg,
        filter::REG_LEGAL_CHANGE_FILTER,
        true,
        Duration::from_secs(1),
    );
    let (sender, receiver) = channel();
    let _ = watcher.watch_async(sender);

    Ok(receiver)
}

fn stop_and_remove_binary(path: &Path) -> io::Result<bool> {
    let s = System::new_all();
    for process in s.processes_by_name(EXE_NAME) {
        if path == process.exe() {
            println!("Found matching process {}", process.name());
            process.kill();
        }
    }

    if path.exists() {
        fs::remove_file(path)?;
        fs::remove_dir(path.parent().expect("Could not find parent directory"))?;
    }

    Ok(true)
}

pub fn run_strip_args(string: &str) -> Option<&str> {
    if string.starts_with('"') {
        let end_quote = string
            .get(1..)
            .expect("Could not strip string")
            .find('"')
            .expect("Could not find end quote");

        string.get(1..end_quote + 1)
    } else {
        Some(string)
    }
}

pub fn install(directory: &Option<PathBuf>, args: &Args) -> io::Result<bool> {
    println!("Installing...");

    let run = RegKey::predef(HKEY_CURRENT_USER).open_subkey(HKCU_STARTUP)?;
    let installed_result: result::Result<String, io::Error> = run.get_value(STARTUP_KEY);

    let appdata: String = std::env::var("APPDATA").expect("No APPDATA directory");
    let mut default_path = PathBuf::from(&appdata);
    default_path.push(INSTALL_DIR);

    let mut dir = PathBuf::clone(match directory {
        Some(dir) => dir,
        _ => &default_path,
    });

    dir.push(EXE_NAME);

    if let Ok(installed) = installed_result {
        let installed_stripped = run_strip_args(installed.as_str())
            .expect("Failed to strip arguments from existing run key");
        println!("Found existing installation ({})", installed_stripped);
        let installed_path = PathBuf::from(installed_stripped);
        if let Ok(exe_path) = env::current_exe() {
            if exe_path.as_path() != installed_path.as_path() {
                stop_and_remove_binary(installed_path.as_path())?;
            }
        }
    }

    let parent_path = dir.parent().expect("Could not find parent folder");
    println!(
        "Creating parent directory if it doesn't exist ({})",
        parent_path.display()
    );
    if !parent_path.exists() {
        fs::create_dir_all(&parent_path)?;
    }

    match env::current_exe() {
        Ok(exe_path) => {
            if exe_path.as_path() != dir.as_path() {
                fs::copy(&exe_path, &dir)?;
                println!("Copied binary {} to {}", exe_path.display(), dir.display());
            }
        }
        Err(e) => panic!("Cannot install: failed to find executable path ({e})"),
    };

    let run_write =
        RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(HKCU_STARTUP, KEY_WRITE)?;
    let mut key = format!("\"{}\"", dir.as_os_str().to_str().unwrap());
    let mut cmd_line = dir.as_os_str().to_str().unwrap().to_string();
    if args.app_only {
        key += " --app-only";
        cmd_line += " --app-only";
    }
    if args.system_only {
        key += " --system-only";
        cmd_line += " --system-only";
    }
    run_write.set_value(STARTUP_KEY, &key)?;

    let mut cmd_line: Vec<u16> = OsStr::new(&cmd_line)
        .encode_wide()
        .chain(iter::once(0u16))
        .collect();

    unsafe {
        let mut sinfo: STARTUPINFOW = mem::zeroed();
        sinfo.cb = mem::size_of::<STARTUPINFOW>() as DWORD;
        sinfo.hStdInput = ptr::null_mut();
        sinfo.hStdOutput = ptr::null_mut();
        sinfo.hStdError = ptr::null_mut();
        sinfo.dwFlags = 0;
        let mut pinfo: PROCESS_INFORMATION = mem::zeroed();

        winapi::um::processthreadsapi::CreateProcessW(
            ptr::null(),
            cmd_line.as_mut_ptr(),
            ptr::null_mut(),
            ptr::null_mut(),
            false as i32,
            CREATE_NO_WINDOW,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut sinfo,
            &mut pinfo,
        );

        println!("Auto dark mode is now running.");
    }

    Ok(true)
}

pub fn uninstall() -> io::Result<bool> {
    println!("Uninstalling...");

    let run = RegKey::predef(HKEY_CURRENT_USER).open_subkey(HKCU_STARTUP)?;
    let installed_result: result::Result<String, io::Error> = run.get_value(STARTUP_KEY);

    match installed_result {
        Ok(installed) => {
            println!("Found installed binary");
            let installed_stripped = run_strip_args(installed.as_str())
                .expect("Failed to strip arguments from existing run key");
            let installed_path = PathBuf::from(installed_stripped);
            println!("Installed path = {}", installed_path.display());
            stop_and_remove_binary(installed_path.as_path())?;
            let run_write = RegKey::predef(HKEY_CURRENT_USER)
                .open_subkey_with_flags(HKCU_STARTUP, KEY_WRITE)?;
            run_write.delete_value(STARTUP_KEY)?;
        }
        _ => {
            println!("Failed to uninstall: app is not installed");
            return Ok(false);
        }
    }

    Ok(true)
}
