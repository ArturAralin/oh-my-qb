use super::value::Value;
use std::{cell::RefCell, rc::Rc};

pub trait Row<'a> {
    fn columns(&self) -> &'static [&'static str];
    fn into_row(self, builder: &mut RowBuilder<'a>);
}

pub struct RowBuilder<'a> {
    bindings: Rc<RefCell<Vec<Value<'a>>>>,
    start: usize,
    end: usize,
}

impl<'a> RowBuilder<'a> {
    pub fn new(bindings: &Rc<RefCell<Vec<Value<'a>>>>) -> Self {
        let start = bindings.as_ref().borrow().len() + 1;

        Self {
            start,
            end: start,
            bindings: Rc::clone(bindings),
        }
    }

    pub fn append_binding(&mut self, value: Value<'a>) {
        self.bindings.as_ref().borrow_mut().push(value);
        self.end += 1;
    }

    pub fn into_slice(self) -> (usize, usize) {
        (self.start, self.end)
    }
}
