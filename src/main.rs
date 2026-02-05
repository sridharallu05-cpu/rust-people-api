use axum::{
    extract::{Json, State},
    http::StatusCode,
    routing::{get, post},
    Router,
};
use dotenvy::dotenv;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{env, sync::{Arc, Mutex}};
use tracing::{info, error};
use tracing_subscriber;

#[derive(Debug, Serialize, Deserialize)]
struct Person {
    name: String,
    age: u32,
}

#[derive(Serialize)]
struct ApiError {
    message: String,
}

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<Connection>>,
}

async fn health() -> &'static str {
    "OK"
}

async fn add_person(
    State(state): State<AppState>,
    Json(person): Json<Person>,
) -> Result<Json<Person>, (StatusCode, Json<ApiError>)> {

    if person.age == 0 || person.age > 120 {
        error!("Invalid age: {}", person.age);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                message: "Age must be between 1 and 120".into(),
            }),
        ));
    }

    let conn = state.db.lock().unwrap();

    conn.execute(
        "INSERT INTO people (name, age) VALUES (?1, ?2)",
        params![person.name, person.age],
    ).unwrap();

    info!("Person added");

    Ok(Json(person))
}

async fn list_people(State(state): State<AppState>) -> Json<Vec<Person>> {
    let conn = state.db.lock().unwrap();

    let mut stmt = conn.prepare("SELECT name, age FROM people").unwrap();

    let people = stmt.query_map([], |row| {
        Ok(Person {
            name: row.get(0)?,
            age: row.get(1)?,
        })
    }).unwrap()
    .map(|p| p.unwrap())
    .collect();

    Json(people)
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    let port = env::var("PORT").unwrap_or("3000".into());
    let db_path = env::var("DB_PATH").unwrap_or("people.db".into());

    info!("Starting server on port {}", port);

    let conn = Connection::open(db_path).unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS people (name TEXT, age INTEGER)",
        [],
    ).unwrap();

    let state = AppState {
        db: Arc::new(Mutex::new(conn)),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/people", post(add_person).get(list_people))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
