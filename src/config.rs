use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub check_for_updates: bool,
    pub slim_path: PathBuf,
    pub slim_args: Vec<String>,
    pub compiler_path: PathBuf,
    pub compiler_args: Vec<String>,
}
