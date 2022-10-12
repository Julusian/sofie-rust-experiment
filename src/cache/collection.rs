use std::collections::HashMap;

use super::doc::DocWithId;

#[derive(Debug, Clone)]
pub enum CacheCollectionError {
    // Unknown(String),
    NotImplemented,
    IsToBeRemoved(&'static str),
    AlreadyExists(String),
    NotFound(String),
    IdMismatch(String),
}

pub struct CollectionDoc<T> {
    pub inserted: bool,
    pub updated: bool,
    pub document: T,
}

type Result<T> = std::result::Result<T, CacheCollectionError>;

pub trait DbCacheReadCollection<T: for<'a> DocWithId<'a>> {
    fn name(&self) -> &str;

    fn find_all(&self) -> Vec<T>;
    fn find_some(&self, cb: fn(doc: &T) -> bool) -> Vec<T>;
    fn find_one_by_id(&self, id: &str) -> Option<T>;
    fn find_one(&self, cb: fn(doc: &T) -> bool) -> Option<T>;
}

pub trait DbCacheWriteCollection<T: for<'a> DocWithId<'a>>: DbCacheReadCollection<T> {
    fn is_modified(&self) -> bool;
    fn mark_for_removal(&mut self);

    fn insert(&mut self, doc: T) -> Result<()>;
    fn remove_by_id(&mut self, id: &str) -> Result<bool>;
    fn remove_by_filter(&mut self, cb: fn(doc: &T) -> bool) -> Result<Vec<String>>;

    fn discard_changes(&mut self);
    fn update_database_with_data(&mut self) -> Result<()>; // TODO

    fn update_one<F: Fn(&T) -> Option<T>>(&mut self, id: &str, cb: F) -> Result<bool>;
    fn update_all<F: Fn(&T) -> Option<T>>(&mut self, cb: F) -> Result<Vec<String>>;

    fn replace_one(&mut self, doc: T) -> Result<bool>;
}

pub struct DbCacheWriteCollectionImpl<T: for<'a> DocWithId<'a>> {
    documents: HashMap<String, Option<CollectionDoc<T>>>,
    documents_raw: Vec<T>,

    is_to_be_removed: bool,

    name: String,
}
impl<T: for<'a> DocWithId<'a>> DbCacheWriteCollectionImpl<T> {
    fn assert_not_to_be_removed(&self, method: &'static str) -> Result<()> {
        if self.is_to_be_removed {
            Err(CacheCollectionError::IsToBeRemoved(method))
        } else {
            Ok(())
        }
    }
}
impl<T: for<'a> DocWithId<'a>> DbCacheReadCollection<T> for DbCacheWriteCollectionImpl<T> {
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
    fn find_some(&self, cb: fn(doc: &T) -> bool) -> Vec<T> {
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
    fn find_one_by_id(&self, id: &str) -> Option<T> {
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
    fn find_one(&self, cb: fn(doc: &T) -> bool) -> Option<T> {
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
impl<T: for<'a> DocWithId<'a>> DbCacheWriteCollection<T> for DbCacheWriteCollectionImpl<T> {
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

    fn insert(&mut self, doc: T) -> Result<()> {
        self.assert_not_to_be_removed("insert")?;

        let id = doc.doc_id();
        let has_existing = self.documents.contains_key(id);
        if has_existing {
            Err(CacheCollectionError::AlreadyExists(id.to_string()))
        } else {
            self.documents.insert(
                id.to_string(),
                Some(CollectionDoc {
                    inserted: !has_existing,
                    updated: has_existing,
                    document: doc,
                }),
            );

            Ok(())
        }
    }

    fn remove_by_id(&mut self, id: &str) -> Result<bool> {
        self.assert_not_to_be_removed("remove_by_id")?;

        if self.documents.contains_key(id) {
            self.documents.insert(id.to_string(), None);

            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn remove_by_filter(&mut self, cb: fn(doc: &T) -> bool) -> Result<Vec<String>> {
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
                    doc.doc_id().to_string(),
                    Some(CollectionDoc {
                        inserted: false,
                        updated: false,
                        document: doc.clone(),
                    }),
                );
            }
        }
    }

    fn update_database_with_data(&mut self) -> Result<()> {
        // TODO
        Err(CacheCollectionError::NotImplemented)
    }

    fn update_one<F: Fn(&T) -> Option<T>>(&mut self, id: &str, cb: F) -> Result<bool> {
        self.assert_not_to_be_removed("update_one")?;

        let doc = self.documents.get_mut(id);
        if let Some(doc) = doc {
            if let Some(doc) = doc {
                let new_doc = cb(&doc.document);
                if let Some(new_doc) = new_doc {
                    if new_doc.doc_id() != id {
                        return Err(CacheCollectionError::IdMismatch(id.to_string()));
                    }

                    // TODO - some equality check?
                    doc.document = new_doc;

                    Ok(true)
                } else {
                    Ok(false)
                }
            } else {
                Err(CacheCollectionError::NotFound(id.to_string()))
            }
        } else {
            Err(CacheCollectionError::NotFound(id.to_string()))
        }
    }

    fn update_all<F: Fn(&T) -> Option<T>>(&mut self, cb: F) -> Result<Vec<String>> {
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

    fn replace_one(&mut self, doc: T) -> Result<bool> {
        self.assert_not_to_be_removed("replace_one")?;

        let id = doc.doc_id();
        let has_existing = self.documents.contains_key(id);

        self.documents.insert(
            id.to_string(),
            Some(CollectionDoc {
                inserted: !has_existing,
                updated: has_existing,
                document: doc,
            }),
        );

        Ok(has_existing)
    }
}
