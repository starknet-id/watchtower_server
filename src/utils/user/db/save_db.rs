use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};

pub async fn save_db(
    db: &Database,
    client_connection_string: String,
    db_name: String,
    db_id: ObjectId,
) -> Result<(), String> {
    // Download client database and store it in the db_saves folder
    // Name it using the db_id and the current date
    // Then save the path in the db_saves collection
    let current_date = chrono::Utc::now();
    let timestamp = current_date.timestamp_millis();
    let out = format!("db_saves/{}/{}", db_id, timestamp.clone());
    let command = format!(
        "mongodump --out {} --gzip --db {} --uri \"{}\"",
        out, db_name, client_connection_string
    );

    let output = std::process::Command::new("cmd")
        .args(&["/C", &command])
        .output();

    if output.is_err() {
        let error = format!("Error while saving db: {}", output.err().unwrap());
        println!("{}", error);
        return Err(error);
    }

    let output = output.unwrap();
    if !output.status.success() {
        let error = format!(
            "Error while saving db: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        println!("{}", error);
        return Err(error);
    }

    let collection: Collection<Document> = db.collection("db_saves");
    let document = doc! {
        "db_id": db_id,
        "time": timestamp,
    };

    collection.insert_one(document, None).await.unwrap();

    return Ok(());
}
