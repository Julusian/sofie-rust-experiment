use std::borrow::Borrow;

use mongodb::{
    bson::{doc, from_document, oid::ObjectId, to_bson, Document},
    error::Result,
    options::UpdateOptions,
};
use serde::Deserialize;

/// Represents an individual update operation for the `bulk_update` function.
#[derive(Debug)]
pub struct BulkUpdate {
    pub query: Document,
    pub update: Document,
    pub options: Option<UpdateOptions>,
}

/// Result of a `bulk_update` operation.
#[derive(Debug, Deserialize)]
pub struct BulkUpdateResult {
    #[serde(rename = "n")]
    pub nb_affected: u64,
    #[serde(rename = "nModified")]
    pub nb_modified: u64,
    #[serde(default)]
    pub upserted: Vec<BulkUpdateUpsertResult>,
}

/// Individual update result of a `bulk_update` operation.
/// Contains the generated id in case of an upsert.
#[derive(Debug, Deserialize)]
pub struct BulkUpdateUpsertResult {
    pub index: u64,
    #[serde(alias = "_id")]
    pub id: ObjectId,
}

pub async fn bulk_update<V, U>(
    collection_name: &str,
    db: &mongodb::Database,
    updates: V,
) -> Result<BulkUpdateResult>
where
    V: Send + Sync + Borrow<Vec<U>>,
    U: Send + Sync + Borrow<BulkUpdate>,
{
    let updates = updates.borrow();
    let mut update_docs = Vec::with_capacity(updates.len());
    for u in updates {
        let u = u.borrow();
        let mut doc = doc! {
            "q": &u.query,
            "u": &u.update,
            "multi": false,
        };
        if let Some(options) = &u.options {
            if let Some(ref upsert) = options.upsert {
                doc.insert("upsert", upsert);
            }
            if let Some(ref collation) = options.collation {
                doc.insert("collation", to_bson(collation)?);
            }
            if let Some(ref array_filters) = options.array_filters {
                doc.insert("arrayFilters", array_filters);
            }
            if let Some(ref hint) = options.hint {
                doc.insert("hint", to_bson(hint)?);
            }
        }
        update_docs.push(doc);
    }
    let mut command = doc! {
        "update": collection_name,
        "updates": update_docs,
    };
    // if let Some(ref write_concern) = self.write_concern() {
    //     command.insert("writeConcern", to_bson(write_concern)?);
    // }
    let res = db.run_command(command, None).await?;
    Ok(from_document(res)?)
}
