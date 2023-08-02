use mongodb::Client;
use std::string;

pub struct CheckDbRes {
    pub success: bool,
    pub message: string::String,
}

pub async fn check_db(client: Client, db_name: String) -> CheckDbRes {
    // Check if the database exists:
    let databases = client.list_database_names(None, None).await.unwrap();
    let mut found = false;
    for name in databases {
        if name == db_name {
            found = true
        }
    }
    if found == false {
        return CheckDbRes {
            success: false,
            message: format!("Database not found: {}", db_name),
        };
    }
    return CheckDbRes {
        success: true,
        message: "".to_string(),
    };
}
