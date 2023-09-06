use super::{super::value::Value, TryIntoArg};
use std::borrow::Cow;

// fn count_bindings(sql: &str) -> usize {
//     let mut count = 0;

//     // support escaping
//     for ch in sql.chars() {
//         if ch == '?' {
//             count += 1;
//         }
//     }

//     count
// }

#[derive(Debug, Clone)]
pub struct Raw<'a> {
    pub sql: Cow<'a, str>,
    pub bindings: Option<Vec<Value<'a>>>,
}

pub trait RawExt<'a> {
    fn raw(self) -> Raw<'a>;
}

impl<'a> RawExt<'a> for &'a str {
    fn raw(self) -> Raw<'a> {
        Raw {
            sql: std::borrow::Cow::Borrowed(self),
            bindings: None,
        }
    }
}

impl<'a> Raw<'a> {
    // todo: support validate

    pub fn bindings(mut self, values: impl IntoIterator<Item = Value<'a>>) -> Self {
        self.bindings = Some(values.into_iter().collect());

        self
    }
}

impl<'a> TryIntoArg<'a> for Raw<'a> {
    type E = crate::error::Error;

    fn try_into_arg(self) -> Result<super::Arg<'a>, Self::E> {
        Ok(super::Arg::Raw(self))
    }
}
