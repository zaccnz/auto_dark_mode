# 🌑 auto dark mode

a simple rust script to change the application theme to dark when the night light is enabled. it runs a lightweight headless application in the background which listens to changes in the windows registry.  
  
written for Windows 10 / 11  

### usage

testing

```
git clone https://github.com/zaccnz/auto_dark_mode.git
cd auto_dark_mode
cargo run
```

installation

```
git clone https://github.com/zaccnz/auto_dark_mode.git
cd auto_dark_mode
cargo run -- --install
```

for a full list of options run `cargo run -- --help` or `auto_dark_mode.exe --help`

```
$ cargo run -- --help
auto_dark_mode 0.1.0
Zac Cleveland
automatically sync dark theme with night light on Windows 10 / 11

USAGE:
    auto_dark_mode.exe [OPTIONS]

OPTIONS:
    -a, --app-only                     Change default app mode only
    -d, --install-dir <INSTALL_DIR>    Directory to copy executable too on install
    -h, --help                         Print help information
    -i, --install                      Copy executable and run on computer start
    -s, --system-only                  Change default Windows mode only
    -u, --uninstall                    Remove this program from your computer
    -V, --version                      Print version information

```

note: please do not rename the executable from `auto_dark_mode.exe`.  

### technologies

- [winreg](https://crates.io/crates/winreg) = "0.5.1"
- [registry](https://crates.io/crates/registry) = "1.2.2"
- [reg-watcher](https://crates.io/crates/reg-watcher) = "0.1"
- [clap](https://crates.io/crates/clap) = "3.0"
- [sysinfo](https://crates.io/crates/sysinfo) = "0.27.0"
- [winapi](https://crates.io/crates/winapi) = "0.3.4"

### references

- [Windows-Auto-Night-Mode](https://github.com/AutoDarkMode/Windows-Auto-Night-Mode)