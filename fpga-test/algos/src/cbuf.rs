pub struct CBuf<T> {
    buf: Box<[T]>,
    pos: usize,
}

impl<T> CBuf<T> {
    pub fn push(&mut self, val: T) -> T {
        let old = std::mem::replace(&mut self.buf[self.pos], val);
        self.pos = (self.pos + 1) % self.len();
        old
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            buf: &self.buf,
            pos: self.pos,
            len: self.len(),
        }
    }
}

impl<T: Clone> Clone for CBuf<T> {
    fn clone(&self) -> Self {
        Self {
            buf: self.buf.clone(),
            pos: self.pos,
        }
    }
}

impl<T> From<Vec<T>> for CBuf<T> {
    fn from(buf: Vec<T>) -> Self {
        assert_ne!(buf.len(), 0, "cannot have zero-length buffer");
        Self {
            buf: buf.into(),
            pos: 0,
        }
    }
}

impl<T> std::iter::FromIterator<T> for CBuf<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from(Vec::from_iter(iter))
    }
}

impl<T> std::ops::Index<usize> for CBuf<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.buf[(self.pos + index) % self.buf.len()]
    }
}

pub struct Iter<'a, T>
where
    T: 'a,
{
    buf: &'a [T],
    pos: usize,
    len: usize,
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }
        let elem = &self.buf[self.pos];
        self.pos = (self.pos + 1) % self.buf.len();
        self.len -= 1;
        Some(elem)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T: 'a> ExactSizeIterator for Iter<'a, T> {}
