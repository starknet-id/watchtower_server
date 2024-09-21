use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};

pub async fn save_db(
    db: &Database,
    client_connection_string: String,
    db_name: String,
    db_id: ObjectId,
    manual: bool,
    mut authentication_database: String,
) -> Result<(), String> {
    // Download client database and store it in the db_saves folder
    // Name it using the db_id and the current date
    // Then save the path in the db_saves collection
    let current_date = chrono::Utc::now();
    let timestamp = current_date.timestamp_millis();
    let out = format!("db_saves/{}/{}", db_id, timestamp.clone());

    if authentication_database == "" {
        authentication_database = "admin".to_string();
    }

    let output = std::process::Command::new("mongodump")
        .args(&[
            "--out",
            &out,
            "--gzip",
            "--authenticationDatabase",
            &authentication_database,
            "--db",
            &db_name,
            "--uri",
            &client_connection_string,
        ])
        .output();

    if output.is_err() {
        let error = format!("Error while saving db: {}", output.err().unwrap());
        return Err(error);
    }

    let output = output.unwrap();
    if !output.status.success() {
        let error = format!(
            "Error while saving db: {}",
            String::from_utf8_lossy(&output.stderr),
        );
        return Err(error);
    }

    let collection: Collection<Document> = db.collection("db_saves");
    let document = doc! {
        "db_id": db_id,
        "time": timestamp,
        "manual": manual,
    };

    collection.insert_one(document, None).await.unwrap();

    return Ok(());
}
