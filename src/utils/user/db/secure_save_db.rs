use mongodb::{
    bson::{doc, oid::ObjectId, Document},
    Collection, Database,
};

use super::save_db::save_db;
use super::update_db_datas::update_db_datas;

pub struct CheckDbRes {
    pub success: bool,
    pub message: String,
}

pub async fn secure_save_db(
    collection: Collection<Document>,
    db: &Database,
    connection_string: String,
    db_name: String,
    db_id: ObjectId,
    manual: bool,
) -> CheckDbRes {
    // Update db status
    collection
        .update_one(
            doc! {"_id": db_id},
            doc! {"$set": {"status": "connecting"}},
            None,
        )
        .await
        .unwrap();
    let res = update_db_datas(
        collection,
        connection_string.to_string(),
        db_name.to_string(),
        db_id.to_hex(),
    )
    .await;
    if res.client_db.is_some() {
        let res2 = save_db(
            db,
            connection_string.to_string(),
            db_name.to_string(),
            db_id,
            manual,
        )
        .await;
        if res2.is_err() {
            return CheckDbRes {
                success: false,
                message: res2.err().unwrap(),
            };
        }
    }
    return CheckDbRes {
        success: res.success,
        message: res.message,
    };
}
