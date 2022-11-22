use crate::Reflect;

/// Iterator that yields key + value pairs.
pub struct PairIter<'a> {
    iter: Box<dyn Iterator<Item = (&'a str, &'a dyn Reflect)> + 'a>,
}

impl<'a> PairIter<'a> {
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a dyn Reflect)> + 'a,
    {
        Self {
            iter: Box::new(iter.into_iter()),
        }
    }
}

impl<'a> Iterator for PairIter<'a> {
    type Item = (&'a str, &'a dyn Reflect);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/// Iterator that yields mutable key + value pairs.
pub struct PairIterMut<'a> {
    iter: Box<dyn Iterator<Item = (&'a str, &'a mut dyn Reflect)> + 'a>,
}

impl<'a> PairIterMut<'a> {
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a str, &'a mut dyn Reflect)> + 'a,
    {
        Self {
            iter: Box::new(iter.into_iter()),
        }
    }
}

impl<'a> Iterator for PairIterMut<'a> {
    type Item = (&'a str, &'a mut dyn Reflect);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/// Iterator that yields single values.
pub struct ValueIter<'a> {
    iter: Box<dyn Iterator<Item = &'a dyn Reflect> + 'a>,
}

impl<'a> ValueIter<'a> {
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a dyn Reflect> + 'a,
    {
        Self {
            iter: Box::new(iter.into_iter()),
        }
    }
}

impl<'a> Iterator for ValueIter<'a> {
    type Item = &'a dyn Reflect;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/// Iterator that yields single mutable values.
pub struct ValueIterMut<'a> {
    iter: Box<dyn Iterator<Item = &'a mut dyn Reflect> + 'a>,
}

impl<'a> ValueIterMut<'a> {
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = &'a mut dyn Reflect> + 'a,
    {
        Self {
            iter: Box::new(iter.into_iter()),
        }
    }
}

impl<'a> Iterator for ValueIterMut<'a> {
    type Item = &'a mut dyn Reflect;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
