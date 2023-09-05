use super::value::Value;

pub trait Row<'a> {
    fn columns() -> &'static [&'static str];
    fn into_row(self, builder: &mut RowBuilder<'a>);
}

#[derive(Debug, Default)]
pub struct RowBuilder<'a> {
    pub values: Vec<Value<'a>>,
}

impl<'a> RowBuilder<'a> {
    pub fn append_binding(&mut self, value: Value<'a>) {
        self.values.push(value);
    }
}
