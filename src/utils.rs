use crate::model::model::StoredConnection;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::{fs, io};

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

pub(crate) fn append_json_to_file<T: Serialize + Clone + for<'de> Deserialize<'de>>(
    data: &T,
    file_path: &str,
) -> std::io::Result<()> {
    // Read the existing file and parse it as a JSON array
    let mut existing_data: Vec<T> = match std::fs::read_to_string(file_path) {
        Ok(file_content) => {
            match serde_json::from_str(&file_content) {
                Ok(data) => data,
                Err(_) => Vec::new(), // If the file is empty or not in JSON array format, create an empty vector
            }
        }
        Err(_) => Vec::new(), // If the file doesn't exist, create an empty vector
    };

    // Add the new data to the existing data
    existing_data.push(data.clone());

    // Serialize the data to a JSON string
    let json_data = serde_json::to_string_pretty(&existing_data).expect("Failed to serialize data");

    // Open a file in write mode
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_path)?;

    // Write the JSON data to the file
    file.write_all(json_data.as_bytes())?;
    Ok(())
}

pub(crate) fn edit_connection_and_save(
    data: &StoredConnection,
    file_path: &str,
    host: &str,
) -> io::Result<()> {
    let mut existing_data: Vec<StoredConnection> = match fs::read_to_string(file_path) {
        Ok(file_content) => serde_json::from_str(&file_content).unwrap_or_else(|_| Vec::new()),
        Err(_) => Vec::new(),
    };

    if let Some(item) = existing_data.iter_mut().find(|item| item.host == host) {
        *item = data.clone();
    } else {
        existing_data.push(data.clone());
    }

    let json_data = serde_json::to_string_pretty(&existing_data).expect("Failed to serialize data");

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_path)?;
    file.write_all(json_data.as_bytes())?;
    Ok(())
}
