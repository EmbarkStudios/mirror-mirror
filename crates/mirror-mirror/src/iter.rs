use crate::Reflect;

/// Iterator that yields key + value pairs.
pub struct PairIter<'a, T = str>
where
    T: ?Sized,
{
    iter: Box<dyn Iterator<Item = (&'a T, &'a dyn Reflect)> + 'a>,
}

impl<'a, T> PairIter<'a, T>
where
    T: ?Sized,
{
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a T, &'a dyn Reflect)> + 'a,
    {
        Self {
            iter: Box::new(iter.into_iter()),
        }
    }
}

impl<'a, T> Iterator for PairIter<'a, T>
where
    T: ?Sized,
{
    type Item = (&'a T, &'a dyn Reflect);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/// Iterator that yields mutable key + value pairs.
pub struct PairIterMut<'a, T = str>
where
    T: ?Sized,
{
    iter: Box<dyn Iterator<Item = (&'a T, &'a mut dyn Reflect)> + 'a>,
}

impl<'a, T> PairIterMut<'a, T>
where
    T: ?Sized,
{
    pub fn new<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (&'a T, &'a mut dyn Reflect)> + 'a,
    {
        Self {
            iter: Box::new(iter.into_iter()),
        }
    }
}

impl<'a, T> Iterator for PairIterMut<'a, T>
where
    T: ?Sized,
{
    type Item = (&'a T, &'a mut dyn Reflect);

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
