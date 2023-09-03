use super::super::value::Value;
use std::borrow::Cow;

fn count_bindings(sql: &str) -> usize {
    let mut count = 0;

    // support escaping
    for ch in sql.chars() {
        if ch == '?' {
            count += 1;
        }
    }

    count
}

#[derive(Debug)]
pub struct Raw<'a> {
    pub sql: Cow<'a, str>,
    pub bindings: Option<Vec<Value<'a>>>,
    bindings_count: usize,
    pub bindings_slice: Option<(usize, usize)>,
}

pub trait RawExt<'a> {
    fn raw(self) -> Raw<'a>;
}

impl<'a> RawExt<'a> for &'a str {
    fn raw(self) -> Raw<'a> {
        Raw {
            sql: std::borrow::Cow::Borrowed(self),
            bindings_count: count_bindings(self),
            bindings: None,
            bindings_slice: None,
        }
    }
}

impl<'a> Raw<'a> {
    pub fn bindings(mut self, values: Vec<Value<'a>>) -> Self {
        if self.bindings_count != values.len() {
            panic!("invalid bindings count")
        }

        self.bindings = Some(values);

        self
    }

    pub fn binding(&mut self, start_idx: usize) -> Option<Vec<Value<'a>>> {
        self.bindings_slice = self
            .bindings
            .as_ref()
            .map(|bindings| (start_idx, start_idx + bindings.len()));

        self.bindings.take()
    }
}
