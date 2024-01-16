use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, SqlitePool};
use std::{env, io::Error, str::FromStr};
use strum::EnumString;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use utoipauto::utoipauto;

#[tokio::main]
async fn main() -> Result<(), Error> {
    #[utoipauto]
    #[derive(OpenApi)]
    #[openapi()]
    struct ApiDoc;

    dotenv::dotenv().expect("Failed to read .env file");

    let pool = SqlitePool::connect(&env::var("DATABASE_URL").expect("connect error"))
        .await
        .expect("Failed to connect");

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/tasks", get(list_tasks))
        .route("/tasks", post(create_task))
        .route("/tasks/:id", put(update_task).delete(delete_task))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8787")
        .await
        .expect("Failed to bind");

    println!(
        "listening onnnn {}",
        listener.local_addr().expect("Failed to get local_addr")
    );
    axum::serve(listener, app.into_make_service()).await
}

#[derive(Deserialize, ToSchema)]
struct CreateTask {
    title: String,
}

#[derive(Deserialize, ToSchema)]
struct UpdateTask {
    title: String,
    status: TaskStatus,
}

#[derive(Serialize, Deserialize, ToSchema, EnumString, sqlx::Type)]
enum TaskStatus {
    Todo,
    Done,
}
impl From<String> for TaskStatus {
    fn from(value: String) -> Self {
        TaskStatus::from_str(value.as_str()).unwrap_or(TaskStatus::Todo)
    }
}

#[derive(Serialize, ToSchema)]
struct Task {
    id: String,
    status: TaskStatus,
    title: String,
    created_at: String,
    updated_at: String,
}

#[utoipa::path(get, path = "/tasks", responses((status = 200, body = [Task])))]
async fn list_tasks(State(pool): State<Pool<Sqlite>>) -> (StatusCode, Json<Vec<Task>>) {
    let tasks = sqlx::query_as!(Task, r#"select * from tasks;"#)
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch");

    (StatusCode::OK, Json(tasks))
}

#[utoipa::path(post, path = "/tasks", responses((status = 201)))]
async fn create_task(
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<CreateTask>,
) -> (StatusCode, Json<Task>) {
    let uuid = uuid::Uuid::new_v4().to_string();
    let task = sqlx::query_as!(
        Task,
        r#" INSERT INTO tasks(id, title) VALUES($1, $2) RETURNING *"#,
        uuid,
        payload.title,
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to insert");

    (StatusCode::CREATED, Json(task))
}

#[utoipa::path(put, path = "/tasks/{id}", responses((status = 200, body = Task)))]
async fn update_task(
    Path(id): Path<String>,
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<UpdateTask>,
) -> (StatusCode, Json<Task>) {
    let task = sqlx::query_as!(
        Task,
        r#"
            UPDATE
                tasks 
            SET
                status = $1,
                title = $2,
                updated_at = (strftime('%Y/%m/%d %H:%M:%S', CURRENT_TIMESTAMP, 'localtime'))
            WHERE
                id = $3 
            RETURNING *;
        "#,
        payload.status,
        payload.title,
        id,
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to update");

    (StatusCode::OK, Json(task))
}

#[utoipa::path(delete, path = "/tasks/{id}", responses((status = 200, body = Task)))]
async fn delete_task(
    Path(id): Path<String>,
    State(pool): State<Pool<Sqlite>>,
) -> (StatusCode, Json<Task>) {
    let task = sqlx::query_as!(Task, r#"DELETE FROM tasks WHERE id = $1 RETURNING *"#, id)
        .fetch_one(&pool)
        .await
        .expect("Failed to delete");

    (StatusCode::OK, Json(task))
}
