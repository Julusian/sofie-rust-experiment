use futures::stream::{TryStream, TryStreamExt};
use futures::{future::LocalBoxFuture, FutureExt, TryFutureExt};
use mongodb::{bson::doc, Collection, Database};
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
        &self,
        query: String, //impl Into<Option<TRaw>>,
        options: Option<String>,
    ) -> LocalBoxFuture<'a, Result<Vec<Doc>, String>>;
    fn find_one_by_id<'a>(
        &'a self,
        id: &'a Id,
        options: Option<String>,
    ) -> LocalBoxFuture<'a, Result<Option<Doc>, String>>;
    fn find_one<'a>(
        &self,
        query: String,
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

    aa: Option<Doc>,
    ai: Option<Id>,
    //
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

            aa: None,
            ai: None,

            collection,
        }
    }
    //     pub fn find_fetch<'a>(
    //         &self,
    //         query: String,
    //         options: Option<String>,
    //     ) -> BoxFuture<'a, Result<Vec<T>, String>> {
    //         todo!()
    //     }
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
        &self,
        query: String, //impl Into<Option<TRaw>>,
        options: Option<String>,
    ) -> LocalBoxFuture<'a, Result<Vec<Doc>, String>> {
        todo!()
    }

    fn find_one_by_id<'a>(
        &'a self,
        id: &'a Id,
        options: Option<String>,
    ) -> LocalBoxFuture<'a, Result<Option<Doc>, String>> {
        Box::pin(async move {
            let mut cursor = self
                .collection
                .find(doc! { "_id": id.unprotect() }, None)
                .await
                .map_err(|_err| format!("query failed"))?;

            let success = cursor.advance().await.unwrap();

            if success {
                let doc = cursor.deserialize_current().unwrap();
                Ok(Some(doc))
            // todo!()
            } else {
                Ok(None)
            }
        })
    }

    fn find_one<'a>(
        &self,
        query: String,
        options: Option<String>,
    ) -> LocalBoxFuture<'a, Result<Option<Doc>, String>> {
        todo!()
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
