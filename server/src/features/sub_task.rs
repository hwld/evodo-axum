pub mod db;
pub mod routes;
pub mod usecases;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use routes::router;

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct ConnectSubTask {
    pub main_task_id: String,
    pub sub_task_id: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct ReconnectSubTask {
    pub old_main_task_id: String,
    pub old_sub_task_id: String,
    pub new_main_task_id: String,
    pub new_sub_task_id: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct DisconnectSubTask {
    pub main_task_id: String,
    pub sub_task_id: String,
}
