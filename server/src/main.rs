use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use http::{header::CONTENT_TYPE, Method};
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, Pool, Sqlite, SqlitePool};
use std::{env, str::FromStr};
use strum::EnumString;
use tower_http::cors::CorsLayer;
use tracing::debug;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use utoipauto::utoipauto;

#[derive(Debug)]
struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
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

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to read .env file");

    tracing_subscriber::fmt::init();

    let pool = SqlitePool::connect(&env::var("DATABASE_URL").expect("connect error"))
        .await
        .expect("Failed to connect");

    #[utoipauto]
    #[derive(OpenApi)]
    #[openapi()]
    struct ApiDoc;

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/tasks", get(list_tasks).post(create_task))
        .route("/tasks/:id", put(update_task).delete(delete_task))
        .route("/task-nodes", get(list_task_nodes).post(create_task_node))
        .route("/task-node-info/:id", put(update_task_node_info))
        .layer(
            CorsLayer::new()
                .allow_origin(["http://localhost:3000".parse().unwrap()])
                .allow_credentials(true)
                .allow_headers([CONTENT_TYPE])
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::HEAD,
                    Method::DELETE,
                    Method::PUT,
                ]),
        )
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8787")
        .await
        .expect("Failed to bind");

    debug!(
        "listening on {:#}",
        listener.local_addr().expect("Failed to get local_adde")
    );
    axum::serve(listener, app).await.unwrap();
}

#[derive(Deserialize, ToSchema, Debug)]
struct CreateTask {
    #[schema(min_length = 1)]
    title: String,
}

#[derive(Deserialize, ToSchema, Debug)]
struct UpdateTask {
    title: String,
    status: TaskStatus,
}

#[derive(Serialize, Deserialize, ToSchema, EnumString, sqlx::Type, Debug)]
enum TaskStatus {
    Todo,
    Done,
}
impl From<String> for TaskStatus {
    fn from(value: String) -> Self {
        TaskStatus::from_str(value.as_str()).unwrap_or(TaskStatus::Todo)
    }
}

#[derive(Serialize, ToSchema, Debug)]
struct Task {
    id: String,
    status: TaskStatus,
    title: String,
    created_at: String,
    updated_at: String,
}

#[tracing::instrument(err)]
#[utoipa::path(get, path = "/tasks", responses((status = 200, body = [Task])))]
async fn list_tasks(
    State(pool): State<Pool<Sqlite>>,
) -> Result<(StatusCode, Json<Vec<Task>>), AppError> {
    let tasks = sqlx::query_as!(Task, r#"select * from tasks;"#)
        .fetch_all(&pool)
        .await?;

    Ok((StatusCode::OK, Json(tasks)))
}

#[derive(Serialize, ToSchema, Debug)]
struct TaskNode {
    task: Task,
    node_info: TaskNodeInfo,
}

#[derive(Serialize, ToSchema, Debug)]
struct TaskNodeInfo {
    id: String,
    task_id: String,
    x: f64,
    y: f64,
}

#[derive(Deserialize, ToSchema, Debug)]
struct CreateTaskNode {
    x: f64,
    y: f64,
    task: CreateTask,
}

#[derive(Deserialize, ToSchema, Debug)]
struct UpdateTaskNodeInfo {
    x: f64,
    y: f64,
}

#[tracing::instrument(err)]
#[utoipa::path(get, path = "/task-nodes", responses((status = 200, body = [TaskNode])))]
async fn list_task_nodes(
    State(pool): State<Pool<Sqlite>>,
) -> Result<(StatusCode, Json<Vec<TaskNode>>), AppError> {
    let records = sqlx::query!(
        // https://docs.rs/sqlx/latest/sqlx/macro.query.html#type-overrides-output-columns
        // ここを見ると、MySQLの場合はONでnot nullのフィールドを比較してたらnon-nullになるっぽいけど、
        // sqliteとpostgresqlではならなそうなので "field!"で上書きする
        r#"
        SELECT
            n.*,
            t.status as "status!",
            t.title as "title!",
            t.created_at as "created_at!",
            t.updated_at as "updated_at!"
        FROM 
            task_node_info as n LEFT JOIN tasks as t
                ON n.task_id = t.id
        "#,
    )
    .fetch_all(&pool)
    .await?;

    let nodes: Vec<TaskNode> = records
        .into_iter()
        .map(|r| TaskNode {
            task: Task {
                id: r.task_id.clone(),
                title: r.title,
                status: r.status.into(),
                created_at: r.created_at,
                updated_at: r.updated_at,
            },
            node_info: TaskNodeInfo {
                id: r.id,
                task_id: r.task_id,
                x: r.x,
                y: r.y,
            },
        })
        .collect();

    Ok((StatusCode::OK, Json(nodes)))
}

#[tracing::instrument(err)]
#[utoipa::path(post, path = "/task-nodes", responses((status = 201, body = TaskNode)))]
async fn create_task_node(
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<CreateTaskNode>,
) -> Result<(StatusCode, Json<TaskNode>), AppError> {
    let mut tx = pool.begin().await?;

    let task_id = uuid::Uuid::new_v4().to_string();
    let task = sqlx::query_as!(
        Task,
        r#" INSERT INTO tasks(id, title) VALUES($1, $2) RETURNING * "#,
        task_id,
        payload.task.title
    )
    .fetch_one(tx.acquire().await?)
    .await?;

    let node_info_id = uuid::Uuid::new_v4().to_string();
    let node_info = sqlx::query_as!(
        TaskNodeInfo,
        r#" INSERT INTO task_node_info(id, task_id, x, y) VALUES($1, $2, $3, $4) RETURNING * "#,
        node_info_id,
        task.id,
        payload.x,
        payload.y
    )
    .fetch_one(tx.acquire().await?)
    .await?;

    tx.commit().await?;

    Ok((StatusCode::OK, Json(TaskNode { task, node_info })))
}

#[tracing::instrument(err)]
#[utoipa::path(put, path = "/task-node-info/{id}", responses((status = 200, body = TaskNodeInfo)))]
async fn update_task_node_info(
    Path(id): Path<String>,
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<UpdateTaskNodeInfo>,
) -> Result<(StatusCode, Json<TaskNodeInfo>), AppError> {
    let task_node_info = sqlx::query_as!(
        TaskNodeInfo,
        r#"
        UPDATE
            task_node_info
        SET
            x = $1,
            y = $2
        WHERE
            id = $3
        RETURNING *;
        "#,
        payload.x,
        payload.y,
        id,
    )
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::OK, Json(task_node_info)))
}

#[tracing::instrument(err)]
#[utoipa::path(post, path = "/tasks", responses((status = 201, body = Task)))]
async fn create_task(
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<CreateTask>,
) -> Result<(StatusCode, Json<Task>), AppError> {
    let uuid = uuid::Uuid::new_v4().to_string();
    let task = sqlx::query_as!(
        Task,
        r#" INSERT INTO tasks(id, title) VALUES($1, $2) RETURNING *"#,
        uuid,
        payload.title,
    )
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(task)))
}

#[tracing::instrument(err)]
#[utoipa::path(put, path = "/tasks/{id}", responses((status = 200, body = Task)))]
async fn update_task(
    Path(id): Path<String>,
    State(pool): State<Pool<Sqlite>>,
    Json(payload): Json<UpdateTask>,
) -> Result<(StatusCode, Json<Task>), AppError> {
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
    .await?;

    Ok((StatusCode::OK, Json(task)))
}

#[tracing::instrument(err)]
#[utoipa::path(delete, path = "/tasks/{id}", responses((status = 200, body = Task)))]
async fn delete_task(
    Path(id): Path<String>,
    State(pool): State<Pool<Sqlite>>,
) -> Result<(StatusCode, Json<Task>), AppError> {
    let task = sqlx::query_as!(Task, r#"DELETE FROM tasks WHERE id = $1 RETURNING *"#, id)
        .fetch_one(&pool)
        .await?;

    Ok((StatusCode::OK, Json(task)))
}
