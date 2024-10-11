use axum::{routing::get, Form, Router};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
type DataMap = Arc<Mutex<HashMap<String, String>>>;

#[tokio::main]
async fn main() {
    // Create a shared data map wrapped in Arc and Mutex
    let data_map: DataMap = Arc::new(Mutex::new(HashMap::new()));

    // Insert some initial data
    {
        let mut map = data_map.lock().expect("Failed to lock data_map");
        map.insert("key1".to_string(), "value1".to_string());
    }

    let app = Router::new()
        .route("/", get(root))
        .route(
            "/push",
            axum::routing::post({
                let data_map = data_map.clone();
                move |iname: Form<Postsheet>| push_data(data_map.clone(), iname)
            }),
        )
        .route(
            "/list",
            get({
                let data_map = data_map.clone();
                move || list_data(data_map.clone())
            }),
        );

    // Run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("Starting server on http://0.0.0.0:3000");
    axum::serve(listener, app).await.unwrap();
}

async fn root() -> String {
    "server running".to_string()
}

#[derive(Deserialize)]
struct Postsheet {
    name: Option<String>,
    missing: Option<String>,
}
async fn push_data(data_map: DataMap, Form(iname): Form<Postsheet>) -> String {
    let mut map = data_map.lock().unwrap();
    map.insert("key2".to_string(), iname.name.clone().unwrap());
    "data pushed".to_string()
}

async fn list_data(data_map: DataMap) -> String {
    let map = data_map.lock().unwrap();
    let mut data = String::new();
    for (key, value) in map.iter() {
        data.push_str(&format!("{}: {}\n", key, value));
    }
    data
}
