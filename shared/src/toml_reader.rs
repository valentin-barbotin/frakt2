use std::fs::File;
use std::io::{self, Read};
use toml::Value;

// fn read_toml_file(file_path: &str) -> io::Result<Value> {
//     let mut file = File::open(file_path)?;
//     let mut toml_string = String::new();
//     file.read_to_string(&mut toml_string)?;
//     let parsed_toml = toml_string.parse::<Value>()?;
//     Ok(parsed_toml)
// }