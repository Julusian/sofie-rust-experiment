use data_model::ids::PartId;
use mongodb::{options::ClientOptions, Client};

use crate::context::direct_collections::{DirectCollections, MongoReadOnlyCollection};

#[macro_use]
extern crate protected_id_derive;
extern crate uuid;

pub mod cache;
mod constants;
pub mod context;
pub mod data_model;
pub mod ingest;
pub mod lib;
pub mod playout;
pub mod protected_id;
// mod types;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    // Parse a connection string into an options struct.
    let mut client_options = ClientOptions::parse("mongodb://localhost:3001")
        .await
        .unwrap();

    // Manually set an option.
    client_options.app_name = Some("Sofie Rust Demo".to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options).unwrap();

    // Get a handle to a database.
    let db = client.database("meteor");

    let collections = DirectCollections::create(&db);

    let part_id = PartId::new_from("W3fAMjHrR6_gqXmzg9z_8PIzAnQ_".to_string());
    let doc = collections
        .parts
        .find_one_by_id(&part_id, None)
        .await
        .unwrap();
}
