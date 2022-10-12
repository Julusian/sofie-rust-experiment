use super::doc::DocWithId;

#[derive(Debug, Clone)]
pub enum CacheObjectError {
    // Unknown(String),
    NotImplemented,
    IsToBeRemoved(&'static str),
}

type Result<T> = std::result::Result<T, CacheObjectError>;

pub trait DbCacheReadObject<T: for<'a> DocWithId<'a>> {
    fn name(&self) -> &str;

    fn doc(&self) -> &T;
}

pub trait DbCacheWriteObject<T: for<'a> DocWithId<'a>>: DbCacheReadObject<T> {
    fn is_modified(&self) -> bool;
    fn mark_for_removal(&mut self);

    fn discard_changes(&mut self);
    fn update_database_with_data(&mut self) -> Result<()>; // TODO

    fn update(&mut self, cb: fn(doc: &T) -> Option<T>) -> Result<bool>;
}

pub struct DbCacheWriteObjectImpl<T: for<'a> DocWithId<'a>> {
    document: T,
    document_raw: T,

    is_to_be_removed: bool,
    updated: bool,

    name: String,
}
impl<T: for<'a> DocWithId<'a>> DbCacheWriteObjectImpl<T> {
    fn assert_not_to_be_removed(&self, method: &'static str) -> Result<()> {
        if self.is_to_be_removed {
            Err(CacheObjectError::IsToBeRemoved(method))
        } else {
            Ok(())
        }
    }
}
impl<T: for<'a> DocWithId<'a>> DbCacheReadObject<T> for DbCacheWriteObjectImpl<T> {
    fn name(&self) -> &str {
        &self.name
    }

    fn doc(&self) -> &T {
        &self.document
    }
}
impl<T: for<'a> DocWithId<'a>> DbCacheWriteObject<T> for DbCacheWriteObjectImpl<T> {
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
    fn update_database_with_data(&mut self) -> Result<()> {
        Err(CacheObjectError::NotImplemented)
    }

    fn update(&mut self, cb: fn(doc: &T) -> Option<T>) -> Result<bool> {
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
