use crate::row::{Row, RowBuilder};
use crate::where_clause::Condition;
use crate::{value::Value, where_clause::Conditions};
use std::{borrow::Cow, cell::RefCell, rc::Rc};

#[derive(Default)]
pub struct QueryBuilder<'a> {
    pub query: Query<'a>,
    pub bindings: Rc<RefCell<Vec<Value<'a>>>>,
}

impl<'a> QueryBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn table(&mut self, table: &'a str) -> &mut Self {
        match &mut self.query {
            Query::Delete(delete) => {
                delete.table = Some(Cow::Borrowed(table));
            }
            Query::Insert(insert) => {
                insert.table = Some(Cow::Borrowed(table));
            }
            Query::Select(select) => {
                select.table = Some(Cow::Borrowed(table));
            }
            Query::Update(update) => {
                update.table = Some(Cow::Borrowed(table));
            }
        };

        self
    }

    pub fn into(&mut self, table: &'a str) -> &mut Self {
        self.table(table)
    }

    pub fn from(&mut self, table: &'a str) -> &mut Self {
        self.table(table)
    }

    pub fn select(&mut self, columns: Option<&'a [&'a str]>) -> &mut Self {
        let columns = columns.map(|columns| {
            columns
                .as_ref()
                .iter()
                .map(|column| Cow::Borrowed(*column))
                .collect::<Vec<_>>()
        });
        self.query = Query::Select(SelectQuery {
            columns,
            table: None,
            where_clause: Default::default(),
        });

        self
    }

    pub fn delete(&mut self) -> &mut Self {
        self.query = Query::Delete(DeleteQuery {
            table: None,
            where_clause: Default::default(),
        });

        self
    }

    pub fn insert(&mut self) -> &mut Self {
        self.query = Query::Insert(InsertQuery {
            table: None,
            rows: Default::default(),
            ordered_columns: Default::default(),
        });

        self
    }

    pub fn update<R: Row<'a>>(&mut self, row: R) -> &mut Self {
        let columns = row.columns();
        let mut builder = RowBuilder::new(&self.bindings);
        row.into_row(&mut builder);

        let (mut start, _) = builder.into_slice();

        let columns = columns
            .iter()
            .map(|col| {
                start += 1;
                (Cow::Borrowed(*col), start)
            })
            .collect::<Vec<_>>();

        self.query = Query::Update(UpdateQuery {
            columns,
            table: None,
            where_clause: Default::default(),
        });

        self
    }

    pub fn value<R: Row<'a>>(&mut self, row: R) -> &mut Self {
        let mut builder = RowBuilder::new(&self.bindings);
        let columns = row.columns();

        row.into_row(&mut builder);

        if let Query::Insert(InsertQuery {
            rows,
            ordered_columns,
            ..
        }) = &mut self.query
        {
            *ordered_columns = Some(columns);
            rows.push(builder.into_slice());
        }

        self
    }

    pub fn values<R: Row<'a>>(&mut self, rows: Vec<R>) -> &mut Self {
        let columns = rows[0].columns();

        rows.into_iter().for_each(|row| {
            let mut builder = RowBuilder::new(&self.bindings);
            row.into_row(&mut builder);

            if let Query::Insert(InsertQuery {
                rows,
                ordered_columns,
                ..
            }) = &mut self.query
            {
                *ordered_columns = Some(columns);
                rows.push(builder.into_slice());
            }
        });

        self
    }
}

impl<'a> Conditions<'a> for QueryBuilder<'a> {
    fn push_cond(&mut self, cond: Condition<'a>) {
        match &mut self.query {
            Query::Select(query) => query.where_clause.push(cond),
            Query::Update(query) => query.where_clause.push(cond),
            Query::Delete(query) => query.where_clause.push(cond),
            _ => {
                unimplemented!("where unsupported yet");
            }
        }
    }

    fn push_bindings<I>(&mut self, values: I)
    where
        I: Iterator<Item = Value<'a>>,
    {
        self.bindings.as_ref().borrow_mut().extend(values);
    }

    fn get_binding_idx(&self) -> usize {
        self.bindings.as_ref().borrow().len() + 1
    }
}

#[derive(Debug, Default)]
pub struct SelectQuery<'a> {
    pub columns: Option<Vec<Cow<'a, str>>>,
    pub table: Option<Cow<'a, str>>,
    pub where_clause: Vec<Condition<'a>>,
}

#[derive(Debug)]
pub struct InsertQuery<'a> {
    pub table: Option<Cow<'a, str>>,
    pub rows: Vec<(usize, usize)>,
    pub ordered_columns: Option<&'static [&'static str]>,
}

#[derive(Debug)]
pub struct UpdateQuery<'a> {
    pub table: Option<Cow<'a, str>>,
    pub columns: Vec<(Cow<'a, str>, usize)>,
    pub where_clause: Vec<Condition<'a>>,
}

#[derive(Debug)]
pub struct DeleteQuery<'a> {
    pub table: Option<Cow<'a, str>>,
    pub where_clause: Vec<Condition<'a>>,
}

#[derive(Debug)]
pub enum Query<'a> {
    Select(SelectQuery<'a>),
    Update(UpdateQuery<'a>),
    Delete(DeleteQuery<'a>),
    Insert(InsertQuery<'a>),
}

impl<'a> Default for Query<'a> {
    fn default() -> Self {
        Self::Select(SelectQuery {
            columns: None,
            table: None,
            where_clause: Default::default(),
        })
    }
}
