use std::sync::Arc;

use futures::TryStreamExt;
use mongodb::{
    bson::{doc, Document},
    Collection,
};

use crate::AppState;

pub async fn clean(app_state: Arc<AppState>) -> bool {
    let services_collection: Collection<Document> = app_state.db.collection("services");
    // Build an array of all the service _ids:
    let mut cursor = services_collection.find(None, None).await.unwrap();
    let mut service_ids: Vec<String> = Vec::new();
    while let Some(service) = cursor.try_next().await.unwrap() {
        service_ids.push(service.get_object_id("_id").unwrap().to_string());
    }
    // Check there is at least 1 service:
    if service_ids.len() == 0 {
        println!("🧹 No services found, skipping clean");
        return false;
    }
    let logs_collection: Collection<Document> = app_state.db.collection("logs");
    // Delete all logs that don't have an app_id in the service_ids array:
    let filter = doc! {
        "app_id": {
            "$nin": service_ids
        }
    };
    let res = logs_collection.delete_many(filter, None).await.unwrap();
    println!("🧹 Deleted {} unlinked logs", res.deleted_count);
    return true;
}
