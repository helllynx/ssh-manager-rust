use std::fs::File;
use std::io::Write;
use serde::Serialize;

pub(crate) fn remove_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<String>()
}


pub(crate) fn write_json_to_file<T: Serialize>(data: &T, file_path: &str) -> std::io::Result<()> {
    // Serialize the data to a JSON string
    let json_data = serde_json::to_string_pretty(data).expect("Failed to serialize data");

    // Open a file in write mode
    let mut file = File::create(file_path)?;

    // Write the JSON data to the file
    file.write_all(json_data.as_bytes())?;
    Ok(())
}