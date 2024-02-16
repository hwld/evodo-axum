pub mod routes;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub use routes::router;

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct ConnectBlockTask {
    pub blocking_task_id: String,
    pub blocked_task_id: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct DisconnectBlockTask {
    pub blocking_task_id: String,
    pub blocked_task_id: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct ReconnectBlockTask {
    pub old_blocking_task_id: String,
    pub old_blocked_task_id: String,
    pub new_blocking_task_id: String,
    pub new_blocked_task_id: String,
}
