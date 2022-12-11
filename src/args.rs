use clap::{Parser, ValueHint};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Copy executable and run on computer start
    #[clap(short, long)]
    pub install: bool,

    /// Directory to copy executable too on install
    #[clap(short = 'd', long, parse(from_os_str), value_hint = ValueHint::FilePath)]
    pub install_dir: Option<PathBuf>,

    /// Remove this program from your computer
    #[clap(short, long)]
    pub uninstall: bool,

    /// Change default app mode only
    #[clap(short, long)]
    pub app_only: bool,

    /// Change default Windows mode only
    #[clap(short, long)]
    pub system_only: bool,
}
