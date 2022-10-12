pub trait DocWithId<'a>: Clone {
    fn doc_id(&'a self) -> &'a str;
}
