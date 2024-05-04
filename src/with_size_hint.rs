pub(crate) struct WithSizeHint<I> {
    inner: I,
    hint: Option<usize>,
}

impl<Item, I: Iterator<Item = Item>> Iterator for WithSizeHint<I> {
    type Item = Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.hint.unwrap_or_default(), self.hint)
    }
}

pub(crate) trait IteratorExt: Iterator + Sized {
    fn with_size_hint(self, hint: Option<usize>) -> WithSizeHint<Self>;
}

impl<Item, I: Iterator<Item = Item>> IteratorExt for I {
    fn with_size_hint(self, hint: Option<usize>) -> WithSizeHint<I> {
        WithSizeHint { inner: self, hint }
    }
}
