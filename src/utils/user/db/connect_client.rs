use std::string;

use mongodb::{options::ClientOptions, Client};

pub async fn connect_client(connection_string: string::String) -> Result<Client, String> {
    let client_options = ClientOptions::parse(connection_string).await;
    if client_options.is_err() {
        return Err(format!(
            "Failed to parse connection string: {}",
            client_options.err().unwrap()
        ));
    }
    let client_options = client_options.unwrap();
    let client = Client::with_options(client_options).unwrap();
    return Ok(client);
}
