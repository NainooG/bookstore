// for me to remember:
    // errors are there because postgres db isn't running
    // to run this: use homebrew to start up postgres again
    // then, connect to "postgres" database using psql postgres
    // then, \l and \dt to list databases then the tables
    // then, use cargo run to run the program
    // then, use postman to test API calls 
    // CELEBRATE!!




#![allow(unused)]

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, patch},
    Json, Router,
};

use serde::{Serialize, Deserialize};
use serde_json::json;

use sqlx::{postgres::PgPoolOptions, PgPool};

use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // 6 steps to create our service:
    // 1. expose environment variables from env file 
    dotenvy::dotenv().expect("Unable to access .env file");

    // 2. set variables from the environment variables

    let server_address = std::env::var("SERVER_ADDRESS").unwrap_or("127.0.0.1:3000".to_owned());
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not found in the env file");


    // 3. create database pool

    let db_pool = PgPoolOptions::new()
        .max_connections(16)
        .connect(&database_url)
        .await
        .expect("Can't connect to database");

    // 4. create our TCP listener

    let listener = TcpListener::bind(server_address)
        .await
        .expect("Could not create TCP Listener");

    println!("listening on {}", listener.local_addr().unwrap());

    // 5. compose the routes

    let app = Router::new()
        .route("/", get( || async { "SENDING IT!!!" }))
        .route("/tasks", get(get_tasks).post(create_task))
        .route("/tasks/:task_id", patch(update_task).delete(delete_task))
        .with_state(db_pool);

    // 6. serve the application

    axum::serve(listener, app)
        .await
        .expect("Error serving application");

}

#[derive(Serialize)]
struct TaskRow {
    task_id: i32,
    name: String,
    priority: Option<i32>,
}

async fn get_tasks(
    State(pg_pool): State<PgPool>
) -> Result<(StatusCode, String), (StatusCode, String)> { 
    let rows = sqlx::query_as!(TaskRow, "SELECT * FROM tasks ORDER BY task_id")
        .fetch_all(&pg_pool)
        .await
        .map_err(|e| { (
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({ "success": false, "message": e.to_string()}).to_string(),
        )})?;

        Ok((
            StatusCode::OK,
            json!({ "success": true, "data": rows }).to_string()
        ))
}

#[derive(Deserialize)]
struct CreateTaskRequest {
    name: String,
    priority: Option<i32>,
}

#[derive(Serialize)]
struct CreateTaskRow {
    task_id: i32,
}

// task is last parameter, any extractor 
// that consumes the body should be the last parameter in an axum handler. read docs.
async fn create_task(
    State(pg_pool): State<PgPool>,
    Json(task): Json<CreateTaskRequest>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let row = sqlx::query_as!(
        CreateTaskRow, "INSERT INTO tasks (name, priority) VALUES ($1, $2) RETURNING task_id",
        task.name,
        task.priority
    ).fetch_one(&pg_pool)
    .await
    .map_err(|e| { (
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({ "success": false, "message": e.to_string()}).to_string(),
    )})?;

    Ok((StatusCode::CREATED, 
        json!({ "success": true, "data": row}).to_string(),
    ))
}

#[derive(Deserialize)]
struct UpdateTaskRequest {
    name: Option<String>,
    priority: Option<i32>,
}

#[axum::debug_handler]
async fn update_task(
    State(pg_pool): State<PgPool>,
    Path(task_id): Path<i32>,
    Json(task): Json<UpdateTaskRequest>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    sqlx::query! (
        "
        UPDATE tasks SET
            name = $2,
            priority = $3
        WHERE task_id = $1
        ",
    task_id, 
    task.name, 
    task.priority
    ).execute(&pg_pool)
    .await
    .map_err(|e| { (
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({ "success": false, "message": e.to_string()}).to_string(),
    )})?;

    Ok((StatusCode::CREATED, 
        json!({ "success": true}).to_string(),
    ))
}


async fn delete_task(
    State(pg_pool): State<PgPool>,
    Path(task_id): Path<i32>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    sqlx::query!(
        "
        DELETE FROM tasks WHERE task_id = $1",
        task_id
    ).execute(&pg_pool)
    .await
    .map_err(|e| { (
        StatusCode::INTERNAL_SERVER_ERROR,
        json!({ "success": false, "message": e.to_string()}).to_string(),
    )})?;

    Ok((StatusCode::CREATED, 
        json!({ "success": true}).to_string(),
    ))
}

