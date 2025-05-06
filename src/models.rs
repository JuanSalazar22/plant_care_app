use serde::{Deserialize, Serialize};
use chrono::NaiveDate;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Plant {
    pub id: Uuid,
    pub name: String,
    #[serde(rename = "wateringFrequencyDays")]
    pub watering_frequency_days: u32,
    #[serde(rename = "fertilizingFrequencyDays")]
    pub fertilizing_frequency_days: u32,
    #[serde(rename = "lastWatered", skip_serializing_if = "Option::is_none")]
    pub last_watered: Option<NaiveDate>,
    #[serde(rename = "lastFertilized", skip_serializing_if = "Option::is_none")]
    pub last_fertilized: Option<NaiveDate>,
    #[serde(rename = "imageFilenames", default)] // Ensure new plants have an empty vec
    pub image_filenames: Vec<String>,
    #[serde(rename = "createdAt")]
    pub created_at: NaiveDate,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AddPlantPayload {
    pub name: String,
    #[serde(rename = "wateringFrequencyDays")]
    pub watering_frequency_days: u32,
    #[serde(rename = "fertilizingFrequencyDays")]
    pub fertilizing_frequency_days: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdatePlantPayload {
    pub name: String,
    #[serde(rename = "wateringFrequencyDays")]
    pub watering_frequency_days: u32,
    #[serde(rename = "fertilizingFrequencyDays")]
    pub fertilizing_frequency_days: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlantTask {
    #[serde(rename = "plantId")]
    pub plant_id: Uuid,
    #[serde(rename = "plantName")]
    pub plant_name: String,
    #[serde(rename = "taskType")]
    pub task_type: String, // "Watering" or "Fertilizing"
    #[serde(rename = "dueDate")]
    pub due_date: NaiveDate,
    #[serde(rename = "daysOverdue", skip_serializing_if = "Option::is_none")]
    pub days_overdue: Option<i64>, // Positive if overdue
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScheduleResponse {
    #[serde(rename = "upcomingTasks")]
    pub upcoming_tasks: Vec<PlantTask>,
    #[serde(rename = "overdueTasks")]
    pub overdue_tasks: Vec<PlantTask>,
}

// --- Helper methods for Plant ---

impl Plant {
    pub fn next_watering_due(&self) -> NaiveDate {
        let last = self.last_watered.unwrap_or(self.created_at);
        last + chrono::Duration::days(self.watering_frequency_days as i64)
    }

    pub fn next_fertilizing_due(&self) -> NaiveDate {
        let last = self.last_fertilized.unwrap_or(self.created_at);
        last + chrono::Duration::days(self.fertilizing_frequency_days as i64)
    }

    /*pub fn is_watering_due_or_overdue(&self, today: NaiveDate) -> bool {
        self.next_watering_due() <= today
    }*/

    /*pub fn is_fertilizing_due_or_overdue(&self, today: NaiveDate) -> bool {
        self.next_fertilizing_due() <= today
    }*/

    pub fn watering_days_overdue(&self, today: NaiveDate) -> Option<i64> {
        let due_date = self.next_watering_due();
        if due_date < today {
            Some((today - due_date).num_days())
        } else {
            None
        }
    }

    pub fn fertilizing_days_overdue(&self, today: NaiveDate) -> Option<i64> {
        let due_date = self.next_fertilizing_due();
        if due_date < today {
            Some((today - due_date).num_days())
        } else {
            None
        }
    }
}