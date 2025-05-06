use crate::models::Plant;
use anyhow::{Context, Result};
use serde_json;
use std::fs::{File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock; // Use tokio's RwLock for async compatibility

pub const DATA_FILE: &str = "data/plants.json";
pub const UPLOADS_DIR: &str = "uploads";

pub async fn load_plants(path: &Path) -> Result<Vec<Plant>> {
    if !path.exists() {
        return Ok(Vec::new()); // Return empty vec if file doesn't exist
    }
    let file = File::open(path).context("Failed to open data file")?;
    let reader = BufReader::new(file);
    let plants: Vec<Plant> =
        serde_json::from_reader(reader).context("Failed to parse JSON data")?;
    Ok(plants)
}

pub async fn save_plants(plants: &Vec<Plant>, path: &Path) -> Result<()> {
    let file = File::create(path).context("Failed to create or open data file for writing")?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, plants)
        .context("Failed to serialize and write JSON data")?;
    Ok(())
}

// Helper to save data within handlers using the AppState structure
pub async fn save_app_state(
    plants_lock: &Arc<RwLock<Vec<Plant>>>,
    data_file_path: &PathBuf,
) -> Result<()> {
    let plants_guard = plants_lock.read().await; // Acquire read lock to clone data
    save_plants(&plants_guard, data_file_path).await
}