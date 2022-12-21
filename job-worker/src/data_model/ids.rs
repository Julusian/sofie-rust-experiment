pub trait ProtectedId {
    fn unprotect(&self) -> &str;
}

#[derive(PartialEq, Clone, Eq, Hash)]
pub struct PartId(String);
impl PartId {
    pub fn new_from(str: String) -> PartId {
        PartId(str)
    }
}
impl ProtectedId for PartId {
    fn unprotect(&self) -> &str {
        &self.0
    }
}

#[derive(PartialEq, Clone, Eq, Hash, PartialOrd, Ord)]
pub struct RundownId(String);
impl RundownId {
    pub fn new_from(str: String) -> RundownId {
        RundownId(str)
    }
}
impl ProtectedId for RundownId {
    fn unprotect(&self) -> &str {
        &self.0
    }
}

#[derive(PartialEq, Clone, Eq, Hash)]
pub struct RundownPlaylistId(String);
impl RundownPlaylistId {
    pub fn new_from(str: String) -> RundownPlaylistId {
        RundownPlaylistId(str)
    }
}
impl ProtectedId for RundownPlaylistId {
    fn unprotect(&self) -> &str {
        &self.0
    }
}

#[derive(PartialEq, Clone, Eq, Hash)]
pub struct RundownPlaylistActivationId(String);
impl RundownPlaylistActivationId {
    pub fn new_from(str: String) -> RundownPlaylistActivationId {
        RundownPlaylistActivationId(str)
    }
}
impl ProtectedId for RundownPlaylistActivationId {
    fn unprotect(&self) -> &str {
        &self.0
    }
}

#[derive(PartialEq, Clone, Eq, Hash)]
pub struct SegmentId(String);
impl SegmentId {
    pub fn new_from(str: String) -> SegmentId {
        SegmentId(str)
    }
}
impl ProtectedId for SegmentId {
    fn unprotect(&self) -> &str {
        &self.0
    }
}

#[derive(PartialEq, Clone, Eq, Hash)]
pub struct PartInstanceId(String);
impl PartInstanceId {
    pub fn new_from(str: String) -> PartInstanceId {
        PartInstanceId(str)
    }
}
impl ProtectedId for PartInstanceId {
    fn unprotect(&self) -> &str {
        &self.0
    }
}

#[derive(PartialEq, Clone, Eq, Hash)]
pub struct SegmentPlayoutId(String);
impl SegmentPlayoutId {
    pub fn new_from(str: String) -> SegmentPlayoutId {
        SegmentPlayoutId(str)
    }
}
impl ProtectedId for SegmentPlayoutId {
    fn unprotect(&self) -> &str {
        &self.0
    }
}

#[derive(PartialEq, Clone, Eq, Hash, PartialOrd, Ord)]
pub struct PieceId(String);
impl PieceId {
    pub fn new_from(str: String) -> PieceId {
        PieceId(str)
    }
}
impl ProtectedId for PieceId {
    fn unprotect(&self) -> &str {
        &self.0
    }
}

#[derive(PartialEq, Clone, Eq, Hash)]
pub struct PieceInstanceId(String);
impl PieceInstanceId {
    pub fn new_from(str: String) -> PieceInstanceId {
        PieceInstanceId(str)
    }
}
impl ProtectedId for PieceInstanceId {
    fn unprotect(&self) -> &str {
        &self.0
    }
}

#[derive(PartialEq, Clone, Eq, Hash)]
pub struct PieceInstanceInfiniteId(String);
impl PieceInstanceInfiniteId {
    pub fn new_from(str: String) -> PieceInstanceInfiniteId {
        PieceInstanceInfiniteId(str)
    }
}
impl ProtectedId for PieceInstanceInfiniteId {
    fn unprotect(&self) -> &str {
        &self.0
    }
}

#[derive(PartialEq, Clone, Eq, Hash)]
pub struct ShowStyleBaseId(String);
impl ShowStyleBaseId {
    pub fn new_from(str: String) -> ShowStyleBaseId {
        ShowStyleBaseId(str)
    }
}
impl ProtectedId for ShowStyleBaseId {
    fn unprotect(&self) -> &str {
        &self.0
    }
}

#[derive(PartialEq, Clone, Eq, Hash)]
pub struct ShowStyleVariantId(String);
impl ShowStyleVariantId {
    pub fn new_from(str: String) -> ShowStyleVariantId {
        ShowStyleVariantId(str)
    }
}
impl ProtectedId for ShowStyleVariantId {
    fn unprotect(&self) -> &str {
        &self.0
    }
}
