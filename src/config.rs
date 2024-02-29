use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub check_for_updates: bool,
    pub slim_path: String,
    pub slim_args: Vec<String>,
    pub compiler_path: String,
    pub compiler_args: Vec<String>,
}
