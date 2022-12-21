// trait ProtectedId2 {
//     fn unprotect(&self) -> String;
// }

#[derive(ProtectedId, PartialEq, Clone, Eq, Hash)]
pub struct PartId {
    #[protected_value]
    id: String,
}

#[derive(ProtectedId, PartialEq, Clone, Eq, Hash, PartialOrd, Ord)]
pub struct RundownId {
    #[protected_value]
    id: String,
}

#[derive(ProtectedId, PartialEq, Clone, Eq, Hash)]
pub struct RundownPlaylistId {
    #[protected_value]
    id: String,
}

#[derive(ProtectedId, PartialEq, Clone, Eq, Hash)]
pub struct RundownPlaylistActivationId {
    #[protected_value]
    id: String,
}

#[derive(ProtectedId, PartialEq, Clone, Eq, Hash)]
pub struct SegmentId {
    #[protected_value]
    id: String,
}

#[derive(ProtectedId, PartialEq, Clone, Eq, Hash)]
pub struct PartInstanceId {
    #[protected_value]
    id: String,
}

#[derive(ProtectedId, PartialEq, Clone, Eq, Hash)]
pub struct SegmentPlayoutId {
    #[protected_value]
    id: String,
}

#[derive(ProtectedId, PartialEq, Clone, Eq, Hash, PartialOrd, Ord)]
pub struct PieceId {
    #[protected_value]
    id: String,
}

#[derive(ProtectedId, PartialEq, Clone, Eq, Hash)]
pub struct PieceInstanceId {
    #[protected_value]
    id: String,
}

#[derive(ProtectedId, PartialEq, Clone, Eq, Hash)]
pub struct PieceInstanceInfiniteId {
    #[protected_value]
    id: String,
}

#[derive(ProtectedId, PartialEq, Clone, Eq, Hash)]
pub struct ShowStyleBaseId {
    #[protected_value]
    id: String,
}

#[derive(ProtectedId, PartialEq, Clone, Eq, Hash)]
pub struct ShowStyleVariantId {
    #[protected_value]
    id: String,
}
