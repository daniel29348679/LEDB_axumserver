use axum::{routing::get, routing::post, Form, Router};
use mysql::prelude::*;
use mysql::*;
use serde::Deserialize;

#[tokio::main]
async fn main() {
    // Create a shared data map wrapped in Arc and Mutex

    let app = Router::new()
        .route("/", get(root))
        .route("/list_all_worker_name", get(list_all_worker_name))
        .route("/add_worker_name", post(add_worker_name))
        .route("/remove_worker_name", post(remove_worker_name))
        .route("/add_mission", post(add_mission))
        .route("/remove_mission", post(remove_mission))
        .route("/list_all_mission", post(list_all_mission))
        .route("/update_mission_state", post(update_mission_state));

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
    worker_name: Option<String>,
    mission_name: Option<String>,
    mission_id: Option<i32>,
    mission_state: Option<String>,
    useless: Option<String>, // This is a hack to allow for empty form submissions
}

async fn list_all_worker_name() -> String {
    get_all_worker_names()
        .iter()
        .fold("".to_string(), |acc, name| format!("{}{}\n", acc, name))
}

fn get_all_worker_names() -> Vec<String> {
    let url = "mysql://root:123456@localhost:3306/todo_list";
    let pool = Pool::new(url).unwrap();
    let mut conn = pool.get_conn().unwrap();
    let query = "SELECT name FROM name_table";
    let names: Vec<String> = conn.query_map(query, |name| name).unwrap();
    names
}

async fn add_worker_name(Form(isheet): Form<Postsheet>) -> String {
    if isheet.worker_name.is_none() {
        return "worker_name not provided".to_string();
    }
    let url = "mysql://root:123456@localhost:3306/todo_list";
    let pool = Pool::new(url).unwrap();
    let mut conn = pool.get_conn().unwrap();
    let query = "INSERT INTO name_table (name) VALUES (:name)";
    match conn.exec_drop(
        query,
        params! { "name" => isheet.worker_name.clone().unwrap() },
    ) {
        Ok(_) => "name insert successful".to_string(),
        Err(_) => "name insert failed".to_string(),
    }
}

async fn remove_worker_name(Form(isheet): Form<Postsheet>) -> String {
    if isheet.worker_name.is_none() {
        return "worker_name not provided".to_string();
    }
    let url = "mysql://root:123456@localhost:3306/todo_list";
    let pool = Pool::new(url).unwrap();
    let mut conn = pool.get_conn().unwrap();
    //DELETE FROM `todo_list`.`name_table` WHERE (`name` = 'Sandy');
    let query = "DELETE FROM name_table WHERE  (`name` =:name)";
    match conn.exec_drop(
        query,
        params! { "name" => isheet.worker_name.clone().unwrap() },
    ) {
        Ok(_) => "worker_name remove successful".to_string(),
        Err(_) => "worker_name remove failed".to_string(),
    }
}

async fn add_mission(Form(isheet): Form<Postsheet>) -> String {
    if isheet.mission_name.is_none() {
        return "mission_name not provided".to_string();
    }
    if isheet.worker_name.is_none() {
        return "worker_name not provided".to_string();
    }
    let url = "mysql://root:123456@localhost:3306/todo_list";
    let pool = Pool::new(url).unwrap();
    let mut conn = pool.get_conn().unwrap();
    let query = "INSERT INTO mission_table (`mission_name`, `worker_name`, `state`) VALUES (:mission_name,:worker_name,:state)";
    match conn.exec_drop(
        query,
        params! { "mission_name" => isheet.mission_name.clone().unwrap(),
        "worker_name" => isheet.worker_name.clone().unwrap(),
        "state" => "nonstart" },
    ) {
        Ok(_) => "mission insert successful".to_string(),
        Err(_) => "mission insert failed".to_string(),
    }
}

async fn remove_mission(Form(isheet): Form<Postsheet>) -> String {
    if isheet.mission_id.is_none() {
        return "mission_id not provided".to_string();
    }
    let url = "mysql://root:123456@localhost:3306/todo_list";
    let pool = Pool::new(url).unwrap();
    let mut conn = pool.get_conn().unwrap();
    let query = "DELETE FROM mission_table WHERE  (`id` =:mission_id)";
    match conn.exec_drop(
        query,
        params! { "mission_id" => isheet.mission_id.clone().unwrap() },
    ) {
        Ok(_) => "mission remove successful".to_string(),
        Err(_) => "mission remove failed".to_string(),
    }
}

async fn list_all_mission(Form(isheet): Form<Postsheet>) -> String {
    get_all_missions(Form(isheet))
        .iter()
        .fold("".to_string(), |acc, mission| {
            format!("{}{}\n", acc, mission.join(" "))
        })
}

fn get_all_missions(Form(isheet): Form<Postsheet>) -> Vec<Vec<String>> {
    let url = "mysql://root:123456@localhost:3306/todo_list";
    let pool = Pool::new(url).unwrap();
    let mut conn = pool.get_conn().unwrap();
    let query = "SELECT * FROM mission_table";

    let missions = conn
        .query_map(query, |n: Row| {
            vec![
                n.get("id").unwrap(),
                n.get("mission_name").unwrap(),
                n.get("worker_name").unwrap(),
                n.get("state").unwrap(),
            ]
        })
        .unwrap();
    if let Some(worker_name) = isheet.worker_name {
        missions
            .iter()
            .filter(|mission| mission[2] == worker_name)
            .map(|mission| mission.clone())
            .collect()
    } else if let Some(mission_name) = isheet.mission_name {
        missions
            .iter()
            .filter(|mission| mission[1] == mission_name)
            .map(|mission| mission.clone())
            .collect()
    } else if let Some(mission_state) = isheet.mission_state {
        missions
            .iter()
            .filter(|mission| mission[3] == mission_state)
            .map(|mission| mission.clone())
            .collect()
    } else if let Some(mission_id) = isheet.mission_id {
        missions
            .iter()
            .filter(|mission| mission[0] == mission_id.to_string())
            .map(|mission| mission.clone())
            .collect()
    } else {
        missions
    }
}

async fn update_mission_state(Form(isheet): Form<Postsheet>) -> String {
    if isheet.mission_id.is_none() {
        return "mission_id not provided".to_string();
    }
    if isheet.mission_state.is_none() {
        return "mission_state not provided".to_string();
    }
    let url = "mysql://root:123456@localhost:3306/todo_list";
    let pool = Pool::new(url).unwrap();
    let mut conn = pool.get_conn().unwrap();
    let query = "UPDATE mission_table SET state = :state WHERE id = :id";
    match conn.exec_drop(
        query,
        params! {
            "state" => isheet.mission_state.clone().unwrap(),
            "id" => isheet.mission_id.clone().unwrap(),
        },
    ) {
        Ok(_) => "mission state update successful".to_string(),
        Err(_) => "mission state update failed".to_string(),
    }
}

async fn update_mission_name(Form(isheet): Form<Postsheet>) -> String {
    if isheet.mission_id.is_none() {
        return "mission_id not provided".to_string();
    }
    if isheet.mission_name.is_none() {
        return "mission_name not provided".to_string();
    }
    let url = "mysql://root:123456@localhost:3306/todo_list";
    let pool = Pool::new(url).unwrap();
    let mut conn = pool.get_conn().unwrap();
    let query = "UPDATE mission_table SET mission_name = :mission_name WHERE id = :id";
    match conn.exec_drop(
        query,
        params! {
            "mission_name" => isheet.mission_name.clone().unwrap(),
            "id" => isheet.mission_id.clone().unwrap(),
        },
    ) {
        Ok(_) => "mission name update successful".to_string(),
        Err(_) => "mission name update failed".to_string(),
    }
}
