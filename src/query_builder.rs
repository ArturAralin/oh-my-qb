use std::borrow::Cow;

#[derive(Debug, Default)]
pub struct SelectQuery<'a> {
    pub columns: Option<Vec<Cow<'a, str>>>,
}

pub struct InsertQuery {
    pub rows: Vec<(usize, usize)>,
    pub ordered_columns: Option<&'static [&'static str]>,
}

pub enum QueryType<'a> {
    Select(SelectQuery<'a>),
    Update,
    Delete,
    Insert(InsertQuery),
}
