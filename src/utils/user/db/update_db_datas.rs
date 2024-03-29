use mongodb::{
    bson::{doc, Document},
    Collection,
};
use std::string;

use super::{check_db::check_db, connect_client::connect_client};

pub struct UpdateDbRes {
    pub success: bool,
    pub message: string::String,
    pub client_db: Option<mongodb::Database>,
}

pub async fn update_db_datas(
    collection: Collection<Document>,
    connection_string: String,
    db_name: String,
    db_id: String,
) -> UpdateDbRes {
    let client = connect_client(connection_string.to_string()).await;
    let mut status;
    let mut message = "".to_string();
    if client.is_err() {
        status = "disconnected";
        message = client.clone().err().unwrap().to_string();
    } else {
        status = "connected";
        let client = client.clone().unwrap();
        let res = check_db(client, db_name.clone()).await;
        if res.success == false {
            status = "disconnected";
            message = res.message;
        }
    }

    let mut collections_list = vec![];
    if status == "connected" {
        // Get database
        let db = client.clone().unwrap().database(&db_name);
        // Get collections
        let collections = db.list_collection_names(None).await.unwrap();
        for collection in collections {
            collections_list.push(collection);
        }
    }

    // Update db status
    let db_object_id = mongodb::bson::oid::ObjectId::parse_str(&db_id).unwrap();
    collection
        .update_one(
            doc! {"_id": db_object_id},
            doc! {"$set": {"status": status
              , "collections": collections_list,
              "message": message.clone()
            }

            },
            None,
        )
        .await
        .unwrap();

    return UpdateDbRes {
        success: if client.is_err() { false } else { true },
        message: message,
        client_db: if client.is_err() {
            None
        } else {
            Some(client.unwrap().database(&db_name))
        },
    };
}
