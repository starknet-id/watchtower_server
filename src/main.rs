mod config;
mod dbconfig;
mod filesconfig;
mod handlers;
mod response;
mod route;
mod structs;
mod userconfig;
mod utils;

use config::Config;
use std::sync::Arc;

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use dotenv::dotenv;
use route::create_router;
use tower_http::cors::CorsLayer;

use mongodb::{options::ClientOptions, Client};

#[derive(Debug)]
pub struct AppState {
    conf: Config,
    db: mongodb::Database,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let config = config::load();

    let database_url = &config.database.url;
    let database_name = &config.database.name;

    // File config
    let configured = filesconfig::config().await;

    if configured != true {
        println!("❌ Failed to configure files");
        return;
    }

    // A Client is needed to connect to MongoDB:
    let client_options = ClientOptions::parse(database_url).await.unwrap();
    let client = Client::with_options(client_options).unwrap();

    let configured = dbconfig::config(config.clone(), client.clone()).await;

    if configured != true {
        println!("❌ Failed to configure database");
        return;
    }

    let db = client.database(database_name);

    println!("🔌 Connected to MongoDB");

    // root user :
    userconfig::config(config.clone(), db.clone()).await;

    let cors = CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>().unwrap())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
            Method::PUT,
        ])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = create_router(Arc::new(AppState {
        db: db,
        conf: config.clone(),
    }))
    .layer(cors);

    println!("🚀 Server started successfully");
    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
