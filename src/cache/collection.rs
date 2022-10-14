use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use super::doc::DocWithId;

#[derive(Debug, Clone)]
pub enum CacheCollectionError<Id: Clone> {
    // Unknown(String),
    NotImplemented,
    IsToBeRemoved(&'static str),
    AlreadyExists(Id),
    NotFound(Id),
    IdMismatch(Id),
}

pub struct CollectionDoc<T> {
    pub inserted: bool,
    pub updated: bool,
    pub document: T,
}

type Result<T, Id> = std::result::Result<T, CacheCollectionError<Id>>;

pub trait DbCacheReadCollection<T: for<'a> DocWithId<'a, Id>, Id: Clone + PartialEq + Eq + Hash> {
    fn name(&self) -> &str;

    fn find_all(&self) -> Vec<T>;
    fn find_some<F: Fn(&T) -> bool>(&self, cb: F) -> Vec<T>;
    fn find_one_by_id(&self, id: &Id) -> Option<T>;
    fn find_one<F: Fn(&T) -> bool>(&self, cb: F) -> Option<T>;
}

pub struct ChangedIds<Id: Clone + PartialEq + Eq + Hash> {
    added: Vec<Id>,
    updated: Vec<Id>,
    removed: Vec<Id>,
    // unchanged: Vec<Id>,
}

pub trait DbCacheWriteCollection<T: for<'a> DocWithId<'a, Id>, Id: Clone + PartialEq + Eq + Hash>:
    DbCacheReadCollection<T, Id>
{
    fn is_modified(&self) -> bool;
    fn mark_for_removal(&mut self);

    fn insert(&mut self, doc: T) -> Result<(), Id>;
    fn remove_by_id(&mut self, id: &Id) -> Result<bool, Id>;
    fn remove_by_filter<F: Fn(&T) -> bool>(&mut self, cb: F) -> Result<Vec<Id>, Id>;

    fn discard_changes(&mut self);
    fn update_database_with_data(&mut self) -> Result<(), Id>;

    fn update_one<F: Fn(&T) -> Option<T>>(&mut self, id: &Id, cb: F) -> Result<bool, Id>;
    fn update_all<F: Fn(&T) -> Option<T>>(&mut self, cb: F) -> Result<Vec<Id>, Id>;

    fn replace_one(&mut self, doc: T) -> Result<bool, Id>;

    fn save_into<F: Fn(&T) -> bool>(
        &mut self,
        filter: F,
        new_data: Vec<T>,
    ) -> Result<ChangedIds<Id>, Id>;
}

pub struct DbCacheWriteCollectionImpl<
    T: for<'a> DocWithId<'a, Id>,
    Id: Clone + PartialEq + Eq + Hash,
> {
    documents: HashMap<Id, Option<CollectionDoc<T>>>,
    documents_raw: Vec<T>,

    is_to_be_removed: bool,

    name: String,
}
impl<T: for<'a> DocWithId<'a, Id>, Id: Clone + PartialEq + Eq + Hash>
    DbCacheWriteCollectionImpl<T, Id>
{
    fn assert_not_to_be_removed(&self, method: &'static str) -> Result<(), Id> {
        if self.is_to_be_removed {
            Err(CacheCollectionError::IsToBeRemoved(method))
        } else {
            Ok(())
        }
    }
}
impl<T: for<'a> DocWithId<'a, Id>, Id: Clone + PartialEq + Eq + Hash> DbCacheReadCollection<T, Id>
    for DbCacheWriteCollectionImpl<T, Id>
{
    fn name(&self) -> &str {
        &self.name
    }

    fn find_all(&self) -> Vec<T> {
        self.documents
            .iter()
            .filter_map(|doc| {
                if let Some(doc) = doc.1 {
                    Some(&doc.document)
                } else {
                    None
                }
            })
            .cloned()
            .collect()
    }
    fn find_some<F: Fn(&T) -> bool>(&self, cb: F) -> Vec<T> {
        let mut res = Vec::new();

        for doc in self.documents.iter() {
            if let Some(doc) = doc.1 {
                if cb(&doc.document) {
                    res.push(doc.document.clone())
                }
            }
        }

        res
    }
    fn find_one_by_id(&self, id: &Id) -> Option<T> {
        let doc = self.documents.get(id);
        if let Some(doc) = doc {
            if let Some(doc) = doc {
                Some(doc.document.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    fn find_one<F: Fn(&T) -> bool>(&self, cb: F) -> Option<T> {
        for doc in self.documents.iter() {
            if let Some(doc) = doc.1 {
                if cb(&doc.document) {
                    return Some(doc.document.clone());
                }
            }
        }

        None
    }
}
impl<T: for<'a> DocWithId<'a, Id>, Id: Clone + PartialEq + Eq + Hash> DbCacheWriteCollection<T, Id>
    for DbCacheWriteCollectionImpl<T, Id>
{
    fn is_modified(&self) -> bool {
        for doc in self.documents.iter() {
            if let Some(doc) = doc.1 {
                if doc.inserted || doc.updated {
                    return true;
                }
            } else {
                return true;
            }
        }
        false
    }

    fn mark_for_removal(&mut self) {
        self.is_to_be_removed = true;
        self.documents.clear();
        self.documents_raw.clear();
    }

    fn insert(&mut self, doc: T) -> Result<(), Id> {
        self.assert_not_to_be_removed("insert")?;

        let id = doc.doc_id();
        let has_existing = self.documents.contains_key(id);
        if has_existing {
            Err(CacheCollectionError::AlreadyExists(id.clone()))
        } else {
            self.documents.insert(
                id.clone(),
                Some(CollectionDoc {
                    inserted: !has_existing,
                    updated: has_existing,
                    document: doc,
                }),
            );

            Ok(())
        }
    }

    fn remove_by_id(&mut self, id: &Id) -> Result<bool, Id> {
        self.assert_not_to_be_removed("remove_by_id")?;

        if self.documents.contains_key(id) {
            self.documents.insert(id.clone(), None);

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn remove_by_filter<F: Fn(&T) -> bool>(&mut self, cb: F) -> Result<Vec<Id>, Id> {
        self.assert_not_to_be_removed("remove_by_filter")?;

        let mut removed = Vec::new();

        for entry in &self.documents {
            if let Some(doc) = entry.1 {
                if cb(&doc.document) {
                    removed.push(entry.0.clone());
                }
            }
        }

        for id in &removed {
            self.documents.remove(id);
        }

        Ok(removed)
    }

    fn discard_changes(&mut self) {
        if self.is_modified() {
            self.documents.clear();

            for doc in &self.documents_raw {
                self.documents.insert(
                    doc.doc_id().clone(),
                    Some(CollectionDoc {
                        inserted: false,
                        updated: false,
                        document: doc.clone(),
                    }),
                );
            }
        }
    }

    fn update_database_with_data(&mut self) -> Result<(), Id> {
        // TODO
        Err(CacheCollectionError::NotImplemented)
    }

    fn update_one<F: Fn(&T) -> Option<T>>(&mut self, id: &Id, cb: F) -> Result<bool, Id> {
        self.assert_not_to_be_removed("update_one")?;

        let doc = self.documents.get_mut(id);
        if let Some(doc) = doc {
            if let Some(doc) = doc {
                let new_doc = cb(&doc.document);
                if let Some(new_doc) = new_doc {
                    if new_doc.doc_id() != id {
                        return Err(CacheCollectionError::IdMismatch(id.clone()));
                    }

                    // TODO - some equality check?
                    doc.document = new_doc;

                    Ok(true)
                } else {
                    Ok(false)
                }
            } else {
                Err(CacheCollectionError::NotFound(id.clone()))
            }
        } else {
            Err(CacheCollectionError::NotFound(id.clone()))
        }
    }

    fn update_all<F: Fn(&T) -> Option<T>>(&mut self, cb: F) -> Result<Vec<Id>, Id> {
        self.assert_not_to_be_removed("update_all")?;

        let mut updated = Vec::new();

        for entry in self.documents.iter_mut() {
            if let Some(doc) = entry.1 {
                let new_doc = cb(&doc.document);
                if let Some(new_doc) = new_doc {
                    if new_doc.doc_id() != entry.0 {
                        return Err(CacheCollectionError::IdMismatch(entry.0.clone()));
                    }

                    // TODO - some equality check?
                    doc.document = new_doc;
                    updated.push(entry.0.clone());
                }
            }
        }

        Ok(updated)
    }

    fn replace_one(&mut self, doc: T) -> Result<bool, Id> {
        self.assert_not_to_be_removed("replace_one")?;

        let id = doc.doc_id();
        let has_existing = self.documents.contains_key(id);

        self.documents.insert(
            id.clone(),
            Some(CollectionDoc {
                inserted: !has_existing,
                updated: has_existing,
                document: doc,
            }),
        );

        Ok(has_existing)
    }

    fn save_into<F: Fn(&T) -> bool>(
        &mut self,
        filter: F,
        new_data: Vec<T>,
    ) -> Result<ChangedIds<Id>, Id> {
        self.assert_not_to_be_removed("save_info")?;

        let docs_matching_filter = self.documents.iter().filter_map(|doc| {
            if let Some(doc) = doc.1 {
                if filter(&doc.document) {
                    Some(doc.document.doc_id().clone())
                } else {
                    None
                }
            } else {
                None
            }
        });

        let mut result = ChangedIds {
            added: Vec::new(),
            updated: Vec::new(),
            removed: Vec::new(),
        };

        let mut docs_to_remove: HashSet<Id> = HashSet::from_iter(docs_matching_filter);

        // Insert new docs;
        for doc in new_data {
            // Mark it as not to remove
            docs_to_remove.remove(doc.doc_id());

            let id = doc.doc_id().clone();

            let was_update = self.replace_one(doc)?;
            if was_update {
                result.updated.push(id);
            } else {
                result.added.push(id);
            }
        }

        // Remove old docs
        for id in docs_to_remove {
            self.remove_by_id(&id);

            result.removed.push(id);
        }

        Ok(result)
    }
}
