use std::{env, path::PathBuf};

pub fn get_exe_directory() -> PathBuf {
    let exe = env::current_exe().expect("Failed to get current executable location");
    exe.parent().expect("Executable must be in some directory").to_owned()
}