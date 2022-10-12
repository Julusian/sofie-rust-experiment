pub trait DocWithId<'a>: Clone {
    fn doc_id(&self) -> &'a str;
}
