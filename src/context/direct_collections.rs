use futures::future::LocalBoxFuture;
use mongodb::Collection;
use std::hash::Hash;

use crate::{
    cache::doc::DocWithId,
    data_model::{
        ids::{PartId, PieceId},
        part::Part,
        piece::Piece,
    },
};

pub trait MongoReadOnlyCollection<
    T: for<'a> DocWithId<'a, Id>,
    TRaw,
    Id: Clone + PartialEq + Eq + Hash,
>
{
    fn name(&self) -> &str;

    fn find_fetch<'a>(
        &self,
        query: String, //impl Into<Option<TRaw>>,
        options: Option<String>,
    ) -> LocalBoxFuture<'a, Result<Vec<T>, String>>;
    fn find_one_by_id<'a>(
        &self,
        id: &Id,
        options: Option<String>,
    ) -> LocalBoxFuture<'a, Result<T, String>>;
    fn find_one<'a>(
        &self,
        query: String,
        options: Option<String>,
    ) -> LocalBoxFuture<'a, Result<T, String>>;
}

pub trait MongoTransform<TLocal, TMongo> {
    fn convert_local_to_mongo(&self, doc: &TLocal) -> TMongo;
    fn convert_mongo_to_local(&self, doc: &TMongo) -> TLocal;
}

pub struct MongoCollectionImpl<
    T: for<'a> DocWithId<'a, Id>,
    RawDoc,
    Id: Clone + PartialEq + Eq + Hash,
> {
    name: String,

    aa: T,
    ai: Id,
    //
    collection: Collection<RawDoc>,
}
// impl<T: for<'a> DocWithId<'a, Id>, RawDoc, Id: Clone + PartialEq + Eq + Hash>
//     MongoCollectionImpl<T, RawDoc, Id>
// {
//     pub fn find_fetch<'a>(
//         &self,
//         query: String,
//         options: Option<String>,
//     ) -> BoxFuture<'a, Result<Vec<T>, String>> {
//         todo!()
//     }
// }
impl<T: for<'a> DocWithId<'a, Id>, TRaw, Id: Clone + PartialEq + Eq + Hash>
    MongoReadOnlyCollection<T, TRaw, Id> for MongoCollectionImpl<T, TRaw, Id>
{
    fn name(&self) -> &str {
        &self.name
    }

    fn find_fetch<'a>(
        &self,
        query: String, //impl Into<Option<TRaw>>,
        options: Option<String>,
    ) -> LocalBoxFuture<'a, Result<Vec<T>, String>> {
        todo!()
    }

    fn find_one_by_id<'a>(
        &self,
        id: &Id,
        options: Option<String>,
    ) -> LocalBoxFuture<'a, Result<T, String>> {
        todo!()
    }

    fn find_one<'a>(
        &self,
        query: String,
        options: Option<String>,
    ) -> LocalBoxFuture<'a, Result<T, String>> {
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
    pub parts: MongoCollectionImpl<Part, Part, PartId>,
    // PartInstances: ICollection<DBPartInstance>
    // PeripheralDevices: ICollection<PeripheralDevice>
    // PeripheralDeviceCommands: ICollection<PeripheralDeviceCommand>
    pub pieces: MongoCollectionImpl<Piece, Piece, PieceId>,
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
