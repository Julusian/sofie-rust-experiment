use std::time::{Duration, Instant};

use chrono::Utc;
use data_model::ids::PartId;
use mongodb::{bson::doc, options::ClientOptions, Client};
use tokio::time::sleep;

use crate::{
    context::{
        context::JobContext,
        direct_collections::{DirectCollections, MongoReadOnlyCollection},
    },
    data_model::ids::{ProtectedId, RundownPlaylistId},
    playout::{cache::PlayoutCache, take::take_next_part_inner},
};

pub mod cache;
mod constants;
pub mod context;
pub mod data_model;
pub mod ingest;
pub mod lib;
pub mod object_with_overrides;
pub mod playout;

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

    let playlist = collections
        .rundown_playlists
        .find_one(doc! { "activationId": { "$exists": true} }, None)
        .await
        .unwrap()
        .expect("No playlist is active!");

    // let part_id = PartId::new_from("W3fAMjHrR6_gqXmzg9z_8PIzAnQ_".to_string());
    // let doc = collections
    //     .parts
    //     .find_one_by_id(&part_id, None)
    //     .await
    //     .unwrap();

    println!("Found playlist {:?}", playlist.id);

    loop {
        let playlist = collections
            .rundown_playlists
            .find_one_by_id(&playlist.id, None)
            .await
            .unwrap()
            .expect("Playlist disappeared");

        if playlist.next_part_instance_id.is_none() {
            break;
        }

        let part_instance = collections
            .part_instances
            .find_one_by_id(&playlist.next_part_instance_id.unwrap(), None)
            .await
            .unwrap()
            .unwrap();

        let before = Instant::now();

        let now = Utc::now();

        let mut cache = PlayoutCache::create(&collections, &playlist.id)
            .await
            .unwrap();

        let context = JobContext::create(collections.clone());

        take_next_part_inner(context, &mut cache, now)
            .await
            .unwrap();

        cache.write_to_database(&collections).await.unwrap();

        println!(
            "{},{:.2?}",
            part_instance.part.id.unprotect(),
            before.elapsed().as_millis()
        );

        sleep(Duration::from_millis(1000)).await;
    }
}
