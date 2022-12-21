use futures::future::LocalBoxFuture;
use mongodb::{
    bson::{doc, Document},
    Collection, Database,
};
use serde::Deserialize;
use std::hash::Hash;

use crate::{
    cache::doc::DocWithId,
    data_model::{
        ids::{PartId, PieceId, ProtectedId},
        part::Part,
        piece::Piece,
    },
};

pub trait MongoReadOnlyCollection<
    Doc: for<'a> DocWithId<'a, Id> + for<'de> Deserialize<'de>,
    Id: Clone + PartialEq + Eq + Hash,
>
{
    fn name(&self) -> &str;

    fn find_fetch<'a>(
        &'a self,
        query: impl Into<Option<Document>> + 'a,
        options: Option<String>,
    ) -> LocalBoxFuture<Result<Vec<Doc>, String>>;
    fn find_one_by_id<'a>(
        &'a self,
        id: &'a Id,
        options: Option<String>,
    ) -> LocalBoxFuture<'a, Result<Option<Doc>, String>>;
    fn find_one<'a>(
        &'a self,
        query: impl Into<Option<Document>> + 'a,
        options: Option<String>,
    ) -> LocalBoxFuture<'a, Result<Option<Doc>, String>>;
}

pub trait MongoTransform<TLocal, TMongo> {
    fn convert_local_to_mongo(&self, doc: &TLocal) -> TMongo;
    fn convert_mongo_to_local(&self, doc: &TMongo) -> TLocal;
}

pub struct MongoCollectionImpl<
    Doc: for<'b> DocWithId<'b, Id> + for<'de> Deserialize<'de>,
    Id: Clone + PartialEq + Eq + Hash,
> {
    name: String,

    ai: Option<Id>,

    collection: Collection<Doc>,
}
impl<
        Doc: for<'a> DocWithId<'a, Id> + for<'de> Deserialize<'de>,
        Id: Clone + PartialEq + Eq + Hash,
    > MongoCollectionImpl<Doc, Id>
{
    pub fn create(db: &Database, name: &str) -> MongoCollectionImpl<Doc, Id> {
        let collection = db.collection::<Doc>(name);

        MongoCollectionImpl {
            name: name.to_string(),

            ai: None,

            collection,
        }
    }

    #[inline]
    fn wrap_mongodb_error<T>(&self, value: mongodb::error::Result<T>) -> Result<T, String> {
        value.map_err(|_err| format!("query failed"))
    }
}
impl<
        Doc: for<'b> DocWithId<'b, Id> + for<'de> Deserialize<'de>,
        Id: Clone + PartialEq + Eq + Hash + ProtectedId,
    > MongoReadOnlyCollection<Doc, Id> for MongoCollectionImpl<Doc, Id>
{
    fn name(&self) -> &str {
        &self.name
    }

    fn find_fetch<'a>(
        &'a self,
        query: impl Into<Option<Document>> + 'a,
        options: Option<String>,
    ) -> LocalBoxFuture<'a, Result<Vec<Doc>, String>> {
        if options.is_some() {
            unimplemented!()
        }

        Box::pin(async move {
            let mut cursor = self.wrap_mongodb_error(self.collection.find(query, None).await)?;

            // TODO - use try_collect() or try_stream() once that is possible without making the docs Send+Sync

            let mut docs = vec![];

            while self.wrap_mongodb_error(cursor.advance().await)? {
                docs.push(self.wrap_mongodb_error(cursor.deserialize_current())?);
            }

            Ok(docs)
        })
    }

    fn find_one_by_id<'a>(
        &'a self,
        id: &'a Id,
        options: Option<String>,
    ) -> LocalBoxFuture<'a, Result<Option<Doc>, String>> {
        self.find_one(doc! { "_id": id.unprotect() }, options)
    }

    fn find_one<'a>(
        &'a self,
        query: impl Into<Option<Document>> + 'a,
        options: Option<String>,
    ) -> LocalBoxFuture<'a, Result<Option<Doc>, String>> {
        if options.is_some() {
            unimplemented!()
        }

        Box::pin(async move {
            let mut cursor = self.wrap_mongodb_error(self.collection.find(query, None).await)?;

            if self.wrap_mongodb_error(cursor.advance().await)? {
                let doc = self.wrap_mongodb_error(cursor.deserialize_current())?;
                Ok(Some(doc))
            } else {
                Ok(None)
            }
        })
    }
}

pub struct DirectCollections {
    // AdLibActions: ICollection<AdLibAction>
    // AdLibPieces: ICollection<AdLibPiece>
    // Blueprints: ICollection<Blueprint>
    // BucketAdLibActions: ICollection<BucketAdLibAction>
    // BucketAdLibPieces: ICollection<BucketAdLib>
    // ExpectedMediaItems: ICollection<ExpectedMediaItem>
    // ExpectedPlayoutItems: ICollection<ExpectedPlayoutItem>
    // IngestDataCache: ICollection<IngestDataCacheObj>
    pub parts: MongoCollectionImpl<Part, PartId>,
    // PartInstances: ICollection<DBPartInstance>
    // PeripheralDevices: ICollection<PeripheralDevice>
    // PeripheralDeviceCommands: ICollection<PeripheralDeviceCommand>
    pub pieces: MongoCollectionImpl<Piece, PieceId>,
    // PieceInstances: ICollection<PieceInstance>
    // Rundowns: ICollection<DBRundown>
    // RundownBaselineAdLibActions: ICollection<RundownBaselineAdLibAction>
    // RundownBaselineAdLibPieces: ICollection<RundownBaselineAdLibItem>
    // RundownBaselineObjects: ICollection<RundownBaselineObj>
    // RundownPlaylists: ICollection<DBRundownPlaylist>
    // Segments: ICollection<DBSegment>
    // ShowStyleBases: ICollection<DBShowStyleBase>
    // ShowStyleVariants: ICollection<DBShowStyleVariant>
    // Studios: ICollection<DBStudio>
    // Timelines: ICollection<TimelineComplete>

    // ExpectedPackages: ICollection<ExpectedPackageDB>
    // PackageInfos: ICollection<PackageInfoDB>

    // ExternalMessageQueue: ICollection<ExternalMessageQueueObj>

    // MediaObjects: ICollection<MediaObjects>
}
impl DirectCollections {
    pub fn create(db: &Database) -> DirectCollections {
        DirectCollections {
            parts: MongoCollectionImpl::create(db, "parts"),
            pieces: MongoCollectionImpl::create(db, "pieces"),
        }
    }
}
