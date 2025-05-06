// src/handlers.rs

// ... (keep imports, AppError, other handlers as they were in the previous corrected version) ...
use crate::models::{
    AddPlantPayload, Plant, PlantTask, ScheduleResponse, UpdatePlantPayload,
};
use crate::utils::{save_app_state};
use crate::AppState;
use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{Utc};
use std::sync::Arc;
use tokio::fs::File as TokioFile;
use tokio::io::AsyncWriteExt;
use tracing::{error, info};
use uuid::Uuid;
use bytes::Bytes; // Import Bytes if using field.chunk()

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!("Application error: {:?}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}


// --- Add Plant ---
pub async fn add_plant(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<AddPlantPayload>,
) -> Result<impl IntoResponse, AppError> {
    let today = Utc::now().date_naive();
    let new_plant = Plant {
        id: Uuid::new_v4(),
        name: payload.name,
        watering_frequency_days: payload.watering_frequency_days,
        fertilizing_frequency_days: payload.fertilizing_frequency_days,
        last_watered: None,
        last_fertilized: None,
        image_filenames: Vec::new(),
        created_at: today,
    };

    let mut plants = state.plants.write().await;
    plants.push(new_plant.clone());
    drop(plants); // Release lock
    let save_result = save_app_state(&state.plants, &state.data_file_path).await;
    save_result?; // Propagate save error if occurred

    info!("Added plant: {}", new_plant.name);
    Ok((StatusCode::CREATED, Json(new_plant)))
}

// --- Get Plants ---
pub async fn get_plants(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Plant>>, AppError> {
    let plants = state.plants.read().await;
    Ok(Json(plants.clone()))
}

// --- Get Plant By ID ---
pub async fn get_plant_by_id(
    State(state): State<Arc<AppState>>,
    Path(plant_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let plants = state.plants.read().await;
    if let Some(plant) = plants.iter().find(|p| p.id == plant_id) {
        Ok((StatusCode::OK, Json(plant.clone())).into_response())
    } else {
        Ok(StatusCode::NOT_FOUND.into_response())
    }
}


// --- Update Plant ---
pub async fn update_plant(
    State(state): State<Arc<AppState>>,
    Path(plant_id): Path<Uuid>,
    Json(payload): Json<UpdatePlantPayload>,
) -> Result<impl IntoResponse, AppError> {
    let mut plants = state.plants.write().await;
    if let Some(plant) = plants.iter_mut().find(|p| p.id == plant_id) {
        plant.name = payload.name;
        plant.watering_frequency_days = payload.watering_frequency_days;
        plant.fertilizing_frequency_days = payload.fertilizing_frequency_days;

        let updated_plant_clone = plant.clone();
        drop(plants); // Release lock
        let save_result = save_app_state(&state.plants, &state.data_file_path).await;
        save_result?; // Propagate save error

        info!("Updated plant: {}", updated_plant_clone.name);
        Ok((StatusCode::OK, Json(updated_plant_clone)).into_response())
    } else {
        Ok(StatusCode::NOT_FOUND.into_response())
    }
}

// --- Delete Plant ---
pub async fn delete_plant(
    State(state): State<Arc<AppState>>,
    Path(plant_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let mut plants = state.plants.write().await;
    let initial_len = plants.len();
    let original_plant_name = plants.iter().find(|p| p.id == plant_id).map(|p| p.name.clone());

    plants.retain(|p| p.id != plant_id);
    let plant_was_removed = plants.len() < initial_len;

    if plant_was_removed {
        if let Some(name) = &original_plant_name {
            info!("Attempting to delete plant: {}", name);
        } else {
            info!("Attempting to delete plant with ID: {}", plant_id);
        }
        drop(plants); // Release lock

        // Save data changes *before* trying to delete directory
        let save_result = save_app_state(&state.plants, &state.data_file_path).await;
        save_result?; // Propagate save error if occurred

        // Now attempt to remove associated images
        let plant_image_dir = state.image_dir_path.join(plant_id.to_string());
        if plant_image_dir.exists() {
            if let Err(e) = tokio::fs::remove_dir_all(&plant_image_dir).await {
                error!("Failed to remove image directory {:?}: {}. Data deletion successful.", plant_image_dir, e);
                // Don't return error here, main goal (data removal) succeeded
            } else {
                info!("Removed image directory for plant {}", plant_id);
            }
        } else {
            info!("Image directory for plant {} not found, nothing to remove.", plant_id);
        }

        info!("Deleted plant data for ID: {}", plant_id);
        Ok(StatusCode::NO_CONTENT.into_response())

    } else {
        drop(plants); // Release lock
        info!("Plant with ID {} not found for deletion.", plant_id);
        Ok(StatusCode::NOT_FOUND.into_response())
    }
}

// --- Mark Watered ---
pub async fn mark_watered(
    State(state): State<Arc<AppState>>,
    Path(plant_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let mut plants = state.plants.write().await;
    if let Some(plant) = plants.iter_mut().find(|p| p.id == plant_id) {
        let today = Utc::now().date_naive();
        plant.last_watered = Some(today);
        let updated_plant_clone = plant.clone();
        drop(plants); // Release lock

        let save_result = save_app_state(&state.plants, &state.data_file_path).await;
        save_result?;

        info!("Marked plant {} as watered", updated_plant_clone.name); // Use cloned name
        Ok((StatusCode::OK, Json(updated_plant_clone)).into_response())
    } else {
        Ok(StatusCode::NOT_FOUND.into_response())
    }
}

// --- Mark Fertilized ---
pub async fn mark_fertilized(
    State(state): State<Arc<AppState>>,
    Path(plant_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let mut plants = state.plants.write().await;
    if let Some(plant) = plants.iter_mut().find(|p| p.id == plant_id) {
        let today = Utc::now().date_naive();
        plant.last_fertilized = Some(today);
        let updated_plant_clone = plant.clone();
        drop(plants); // Release lock

        let save_result = save_app_state(&state.plants, &state.data_file_path).await;
        save_result?;

        info!("Marked plant {} as fertilized", updated_plant_clone.name); // Use cloned name
        Ok((StatusCode::OK, Json(updated_plant_clone)).into_response())
    } else {
        Ok(StatusCode::NOT_FOUND.into_response())
    }
}

// --- Get Schedule ---
pub async fn get_schedule(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ScheduleResponse>, AppError> {
    let plants = state.plants.read().await;
    let today = Utc::now().date_naive();
    let three_days_later = today + chrono::Duration::days(3);

    let mut upcoming_tasks = Vec::new();
    let mut overdue_tasks = Vec::new();

    for plant in plants.iter() {
        // Check Watering
        let next_watering = plant.next_watering_due();
        if next_watering < today {
            overdue_tasks.push(PlantTask {
                plant_id: plant.id,
                plant_name: plant.name.clone(),
                task_type: "Watering".to_string(),
                due_date: next_watering,
                days_overdue: plant.watering_days_overdue(today),
            });
        } else if next_watering >= today && next_watering <= three_days_later {
            upcoming_tasks.push(PlantTask {
                plant_id: plant.id,
                plant_name: plant.name.clone(),
                task_type: "Watering".to_string(),
                due_date: next_watering,
                days_overdue: None,
            });
        }

        // Check Fertilizing
        let next_fertilizing = plant.next_fertilizing_due();
        if next_fertilizing < today {
            overdue_tasks.push(PlantTask {
                plant_id: plant.id,
                plant_name: plant.name.clone(),
                task_type: "Fertilizing".to_string(),
                due_date: next_fertilizing,
                days_overdue: plant.fertilizing_days_overdue(today),
            });
        } else if next_fertilizing >= today && next_fertilizing <= three_days_later {
            upcoming_tasks.push(PlantTask {
                plant_id: plant.id,
                plant_name: plant.name.clone(),
                task_type: "Fertilizing".to_string(),
                due_date: next_fertilizing,
                days_overdue: None,
            });
        }
    }

    upcoming_tasks.sort_by_key(|t| t.due_date);
    overdue_tasks.sort_by_key(|t| t.due_date);

    Ok(Json(ScheduleResponse {
        upcoming_tasks,
        overdue_tasks,
    }))
}

// --- CORRECTED upload_image Handler ---
pub async fn upload_image(
    State(state): State<Arc<AppState>>,
    Path(plant_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let plant_exists = {
        let plants_read = state.plants.read().await;
        plants_read.iter().any(|p| p.id == plant_id)
    };

    if !plant_exists {
        info!("Upload image failed: Plant ID {} not found", plant_id);
        return Ok(StatusCode::NOT_FOUND.into_response());
    }

    let plant_image_dir = state.image_dir_path.join(plant_id.to_string());
    tokio::fs::create_dir_all(&plant_image_dir).await.map_err(|e| {
        error!("Failed to create image directory {:?}: {}", plant_image_dir, e);
        AppError::from(anyhow::anyhow!("Failed to create image directory: {}", e))
    })?;

    let mut uploaded_filename: Option<String> = None;

    // *** FIX IS HERE: Apply '?' directly to next_field().await ***
    while let Ok(Some(field_result)) = multipart.next_field().await {
        // Use '?' to extract the Field or propagate AppError
        // This works because MultipartError -> axum::Error -> anyhow::Error -> AppError
        let mut field = field_result; // <-- The fix!

        let name = field.name().unwrap_or("").to_string();

        if name == "plantImage" {
            let original_filename = field.file_name().unwrap_or("").to_string();

            // Check if it seems like a file based on non-empty filename
            if !original_filename.is_empty() {
                let extension = std::path::Path::new(&original_filename)
                    .extension()
                    .and_then(std::ffi::OsStr::to_str)
                    .unwrap_or("bin");

                let new_filename = format!("{}.{}", Uuid::new_v4(), extension);
                let filepath = plant_image_dir.join(&new_filename);

                info!("Receiving file '{}', saving to '{}'", original_filename, filepath.display());

                let mut file = TokioFile::create(&filepath).await.map_err(|e| {
                    error!("Failed to create file '{}': {}", filepath.display(), e);
                    AppError::from(anyhow::anyhow!("Failed to create file for saving: {}", e))
                })?;

                // Stream data chunk by chunk
                while let Ok(Some(chunk_result)) = field.chunk().await {
                    // Use '?' to get Bytes or propagate AppError
                    // This works because axum::Error -> anyhow::Error -> AppError
                    let chunk: Bytes = chunk_result; // <-- Apply '?' here too

                    let _ = file.write_all(&chunk).await.map_err(async |e| {
                        error!("Failed to write chunk to '{}': {}", filepath.display(), e);
                        let _ = tokio::fs::remove_file(&filepath).await; // Attempt cleanup
                        AppError::from(anyhow::anyhow!("Failed to write upload chunk: {}", e))
                    });
                }

                file.flush().await.map_err(|e| {
                    error!("Failed to flush file '{}': {}", filepath.display(), e);
                    AppError::from(anyhow::anyhow!("Failed to flush saved file: {}", e))
                })?;

                uploaded_filename = Some(new_filename);
                info!("Successfully saved file '{}'", filepath.display());
                break;
            } else {
                info!("Field 'plantImage' received, but has no filename. Skipping.");
                // Consume the field data to allow multipart processing to continue
                let _ = field.bytes().await;
            }
        } else {
            info!("Ignoring non-'plantImage' field: {}", name);
            // Consume the field data
            let _ = field.bytes().await;
        }
    } // End while loop over fields

    if let Some(filename) = uploaded_filename {
        let mut plants = state.plants.write().await;
        if let Some(plant) = plants.iter_mut().find(|p| p.id == plant_id) {
            plant.image_filenames.push(filename.clone());
            let updated_plant_clone = plant.clone();
            drop(plants); // Release lock

            let save_result = save_app_state(&state.plants, &state.data_file_path).await;
            save_result?;

            info!("Added image '{}' to plant '{}'", filename, updated_plant_clone.name);
            return Ok((StatusCode::OK, Json(updated_plant_clone)).into_response());
        } else {
            error!("Plant {} disappeared during image upload processing after file save", plant_id);
            let orphaned_filepath = state.image_dir_path.join(plant_id.to_string()).join(&filename);
            let _ = tokio::fs::remove_file(&orphaned_filepath).await; // Attempt cleanup
            return Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response());
        }
    } else {
        info!("Upload request completed for plant {}, but no valid 'plantImage' file field was found or processed.", plant_id);
        return Ok((StatusCode::BAD_REQUEST, "No valid image file found in 'plantImage' field").into_response());
    }
}