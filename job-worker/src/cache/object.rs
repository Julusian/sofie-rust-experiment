use mongodb::{bson::doc, options::ReplaceOptions};
use serde::{Deserialize, Serialize};

use crate::{context::direct_collections::MongoCollectionImpl, data_model::ids::ProtectedId};

use super::doc::DocWithId;
use core::hash::Hash;

#[derive(Debug, Clone)]
pub enum CacheObjectError {
    // Unknown(String),
    NotImplemented,
    IsToBeRemoved(&'static str),
}

type Result<T> = std::result::Result<T, CacheObjectError>;

pub trait DbCacheReadObject<
    T: for<'a> DocWithId<'a, Id> + for<'de> Deserialize<'de> + Serialize,
    Id: Clone + PartialEq + Eq + Hash + ProtectedId,
>
{
    fn name(&self) -> &str;

    fn doc_id(&self) -> &Id;

    fn doc(&self) -> &T;
}

pub trait DbCacheWriteObject<
    T: for<'a> DocWithId<'a, Id> + for<'de> Deserialize<'de> + Serialize,
    Id: Clone + PartialEq + Eq + Hash + ProtectedId,
>: DbCacheReadObject<T, Id>
{
    fn is_modified(&self) -> bool;
    fn mark_for_removal(&mut self);

    fn discard_changes(&mut self);

    fn update<F: Fn(&T) -> Option<T>>(&mut self, cb: F) -> Result<bool>;
}

pub struct DbCacheWriteObjectImpl<
    T: for<'a> DocWithId<'a, Id> + for<'de> Deserialize<'de> + Serialize,
    Id: Clone + PartialEq + Eq + Hash + ProtectedId,
> {
    id: Id,

    document: T,
    document_raw: T,

    is_to_be_removed: bool,
    updated: bool,

    name: String,
}
impl<
        T: for<'a> DocWithId<'a, Id> + for<'de> Deserialize<'de> + Serialize,
        Id: Clone + PartialEq + Eq + Hash + ProtectedId,
    > DbCacheWriteObjectImpl<T, Id>
{
    pub fn from_document(collection_name: String, doc: T) -> DbCacheWriteObjectImpl<T, Id> {
        DbCacheWriteObjectImpl {
            id: doc.doc_id().clone(),

            document: doc.clone(),
            document_raw: doc,

            is_to_be_removed: false,
            updated: false,

            name: collection_name,
        }
    }

    fn assert_not_to_be_removed(&self, method: &'static str) -> Result<()> {
        if self.is_to_be_removed {
            Err(CacheObjectError::IsToBeRemoved(method))
        } else {
            Ok(())
        }
    }

    pub async fn save_into_collection(
        &mut self,
        collection: &MongoCollectionImpl<T, Id>,
    ) -> std::result::Result<(), String> {
        if !self.is_to_be_removed && self.updated {
            let options = ReplaceOptions::builder().upsert(true).build();

            let err = collection
                .collection
                .replace_one(
                    doc! {"_id": self.doc_id().unprotect() },
                    &self.document,
                    options,
                )
                .await;

            collection.wrap_mongodb_error(err)?;
        }

        Ok(())
    }
}
impl<
        T: for<'a> DocWithId<'a, Id> + for<'de> Deserialize<'de> + Serialize,
        Id: Clone + PartialEq + Eq + Hash + ProtectedId,
    > DbCacheReadObject<T, Id> for DbCacheWriteObjectImpl<T, Id>
{
    fn name(&self) -> &str {
        &self.name
    }

    fn doc_id(&self) -> &Id {
        &self.id
    }

    fn doc(&self) -> &T {
        &self.document
    }
}
impl<
        T: for<'a> DocWithId<'a, Id> + for<'de> Deserialize<'de> + Serialize,
        Id: Clone + PartialEq + Eq + Hash + ProtectedId,
    > DbCacheWriteObject<T, Id> for DbCacheWriteObjectImpl<T, Id>
{
    fn is_modified(&self) -> bool {
        self.updated
    }

    fn mark_for_removal(&mut self) {
        self.is_to_be_removed = true;
    }

    fn discard_changes(&mut self) {
        if self.updated {
            self.updated = false;
            self.document = self.document_raw.clone();
        }
    }

    fn update<F: Fn(&T) -> Option<T>>(&mut self, cb: F) -> Result<bool> {
        self.assert_not_to_be_removed("update")?;

        let new_doc = cb(&self.document);
        if let Some(new_doc) = new_doc {
            // TODO - some equality check?

            self.updated = true;
            self.document = new_doc;

            Ok(true)
        } else {
            Ok(false)
        }
    }
}
