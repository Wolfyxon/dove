use std::path::{Path, PathBuf};

pub fn get_dir() -> PathBuf {
    dirs::config_dir().map(|v| {
        v.join("Wolfyxon/dove")
    }).unwrap_or(
        Path::new("dove").to_path_buf()
    )
}

pub fn get_token_file_path() -> PathBuf {
    get_dir().join("DO_NOT_SHARE.dat") 
}
