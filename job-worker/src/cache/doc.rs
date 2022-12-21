use std::hash::Hash;

pub trait DocWithId<'a, Id: Clone + PartialEq + Eq + Hash>: Clone {
    fn doc_id(&'a self) -> &'a Id;
}
