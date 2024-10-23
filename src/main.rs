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

    let app = Router::new().route("/", get( || async { "Hello World " }));

    // 6. serve the application

    axum::serve(listener, app)
        .await
        .expect("Error serving application");

}