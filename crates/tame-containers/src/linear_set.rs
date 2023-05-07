// This code originally from the `linear_map` crate, but elected to be copy/reimplemented with modifications here as
// that crate is no longer actively maintained.

use core::cmp::Ordering;
use core::mem::ManuallyDrop;
use std::borrow::Borrow;
use std::fmt;
use std::iter::{Chain, FromIterator};
use std::ops::{BitAnd, BitOr, BitXor, Sub};

use super::linear_map::{Keys, LinearMap};

/// An implementation of a set using the underlying representation of a
/// [`LinearMap`] where the value is (). This means the set is effectively
/// a [`Vec`] with operations optimized for working on items that function as
/// a set.
/// 
/// # Ordering
/// 
/// Ordering of elements is well-defined but may be unexpected due to the use of
/// [`swap_remove`][Vec::swap_remove] in the implementation of [`LinearMap::remove`].
/// If you want to maintain order on removal, see [`LinearMap::shift_remove`], but note
/// its additional time complexity. You can also use one of the sorting functions to
/// change the order after doing arbitrary insertion and removals.
///
/// # Examples
///
/// ```
/// use tame_containers::linear_set::LinearSet;;
/// // Type inference lets us omit an explicit type signature (which
/// // would be `LinearSet<&str>` in this example).
/// let mut books = LinearSet::new();
///
/// // Add some books.
/// books.insert("A Dance With Dragons");
/// books.insert("To Kill a Mockingbird");
/// books.insert("The Odyssey");
/// books.insert("The Great Gatsby");
///
/// // Check for a specific one.
/// if !books.contains("The Winds of Winter") {
///     println!("We have {} books, but The Winds of Winter ain't one.",
///              books.len());
/// }
///
/// // Remove a book.
/// books.remove("The Odyssey");
///
/// // Iterate over everything.
/// for book in &books {
///     println!("{}", book);
/// }
/// ```
///
/// The easiest way to use `LinearSet` with a custom type is to derive
/// `Eq`. We must also derive `PartialEq`, this will in the
/// future be implied by `Eq`.
///
/// ```
/// use tame_containers::linear_set::LinearSet;;
/// #[derive(Eq, PartialEq, Debug)]
/// struct Viking<'a> {
///     name: &'a str,
///     power: usize,
/// }
///
/// let mut vikings = LinearSet::new();
///
/// vikings.insert(Viking { name: "Einar", power: 9 });
/// vikings.insert(Viking { name: "Einar", power: 9 });
/// vikings.insert(Viking { name: "Olaf", power: 4 });
/// vikings.insert(Viking { name: "Harald", power: 8 });
///
/// // Use derived implementation to print the vikings.
/// for x in &vikings {
///     println!("{:?}", x);
/// }
/// ```
#[derive(Clone)]
pub struct LinearSet<T> {
    map: LinearMap<T, ()>,
}

impl<T: Eq> LinearSet<T> {
    /// Creates an empty LinearSet.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    /// let mut set: LinearSet<i32> = LinearSet::new();
    /// ```
    #[inline]

    pub fn new() -> LinearSet<T> {
        LinearSet {
            map: LinearMap::new(),
        }
    }

    /// Creates an empty LinearSet with space for at least `n` elements in
    /// the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    /// let mut set: LinearSet<i32> = LinearSet::with_capacity(10);
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> LinearSet<T> {
        LinearSet {
            map: LinearMap::with_capacity(capacity),
        }
    }
}

impl<T> LinearSet<T>
where
    T: Eq,
{
    /// Returns the number of elements the set can hold without reallocating.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    /// let set: LinearSet<i32> = LinearSet::with_capacity(100);
    /// assert!(set.capacity() >= 100);
    /// ```
    #[inline]

    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the `LinearSet`. The collection may reserve more space to avoid
    /// frequent reallocations.
    ///
    /// # Panics
    ///
    /// Panics if the new allocation size overflows `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    /// let mut set: LinearSet<i32> = LinearSet::new();
    /// set.reserve(10);
    /// ```

    pub fn reserve(&mut self, additional: usize) {
        self.map.reserve(additional)
    }

    /// Shrinks the capacity of the set as much as possible. It will drop
    /// down as much as possible while maintaining the internal rules
    /// and possibly leaving some space in accordance with the resize policy.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    ///
    /// let mut set = LinearSet::with_capacity(100);
    /// set.insert(1);
    /// set.insert(2);
    /// assert!(set.capacity() >= 100);
    /// set.shrink_to_fit();
    /// assert!(set.capacity() >= 2);
    /// ```

    pub fn shrink_to_fit(&mut self) {
        self.map.shrink_to_fit()
    }

    /// An iterator visiting all elements.
    /// 
    /// Order is well-defined but may be different than expected.
    /// See notes on [ordering][LinearSet#Ordering].
    /// 
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    /// let mut set = LinearSet::new();
    /// set.insert("a");
    /// set.insert("b");
    ///
    /// for x in set.iter() {
    ///     println!("{}", x);
    /// }
    /// ```

    pub fn iter(&self) -> Iter<T> {
        Iter {
            iter: self.map.keys(),
        }
    }

    /// Visit the values representing the difference.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    /// let a: LinearSet<_> = [1, 2, 3].iter().cloned().collect();
    /// let b: LinearSet<_> = [4, 2, 3, 4].iter().cloned().collect();
    ///
    /// // Can be seen as `a - b`.
    /// for x in a.difference(&b) {
    ///     println!("{}", x); // Print 1
    /// }
    ///
    /// let diff: LinearSet<_> = a.difference(&b).cloned().collect();
    /// assert_eq!(diff, [1].iter().cloned().collect());
    ///
    /// // Note that difference is not symmetric,
    /// // and `b - a` means something else:
    /// let diff: LinearSet<_> = b.difference(&a).cloned().collect();
    /// assert_eq!(diff, [4].iter().cloned().collect());
    /// ```

    pub fn difference<'a>(&'a self, other: &'a LinearSet<T>) -> Difference<'a, T> {
        Difference {
            iter: self.iter(),
            other,
        }
    }

    /// Visit the values representing the symmetric difference.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    /// let a: LinearSet<_> = [1, 2, 3].iter().cloned().collect();
    /// let b: LinearSet<_> = [4, 2, 3, 4].iter().cloned().collect();
    ///
    /// for x in a.symmetric_difference(&b) {
    ///     println!("{}", x);
    /// }
    ///
    /// let diff1: LinearSet<_> = a.symmetric_difference(&b).cloned().collect();
    /// let diff2: LinearSet<_> = b.symmetric_difference(&a).cloned().collect();
    ///
    /// assert_eq!(diff1, diff2);
    /// assert_eq!(diff1, [1, 4].iter().cloned().collect());
    /// ```

    pub fn symmetric_difference<'a>(
        &'a self,
        other: &'a LinearSet<T>,
    ) -> SymmetricDifference<'a, T> {
        SymmetricDifference {
            iter: self.difference(other).chain(other.difference(self)),
        }
    }

    /// Visit the values representing the intersection.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    /// let a: LinearSet<_> = [1, 2, 3].iter().cloned().collect();
    /// let b: LinearSet<_> = [4, 2, 3, 4].iter().cloned().collect();
    ///
    /// for x in a.intersection(&b) {
    ///     println!("{}", x);
    /// }
    ///
    /// let intersection: LinearSet<_> = a.intersection(&b).cloned().collect();
    /// assert_eq!(intersection, [2, 3].iter().cloned().collect());
    /// ```

    pub fn intersection<'a>(&'a self, other: &'a LinearSet<T>) -> Intersection<'a, T> {
        Intersection {
            iter: self.iter(),
            other,
        }
    }

    /// Visit the values representing the union.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    /// let a: LinearSet<_> = [1, 2, 3].iter().cloned().collect();
    /// let b: LinearSet<_> = [4, 2, 3, 4].iter().cloned().collect();
    ///
    /// for x in a.union(&b) {
    ///     println!("{}", x);
    /// }
    ///
    /// let union: LinearSet<_> = a.union(&b).cloned().collect();
    /// assert_eq!(union, [1, 2, 3, 4].iter().cloned().collect());
    /// ```

    pub fn union<'a>(&'a self, other: &'a LinearSet<T>) -> Union<'a, T> {
        Union {
            iter: self.iter().chain(other.difference(self)),
        }
    }

    /// Returns the number of elements in the set.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    ///
    /// let mut v = LinearSet::new();
    /// assert_eq!(v.len(), 0);
    /// v.insert(1);
    /// assert_eq!(v.len(), 1);
    /// ```

    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if the set contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    ///
    /// let mut v = LinearSet::new();
    /// assert!(v.is_empty());
    /// v.insert(1);
    /// assert!(!v.is_empty());
    /// ```

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Clears the set, returning all elements in an iterator.
    #[inline]
    pub fn drain(&mut self) -> Drain<T> {
        Drain {
            iter: self.map.drain(),
        }
    }

    /// Clears the set, removing all elements. 
    /// 
    /// Unlike the hashtable-based sets provided by this crate, keeps the
    /// whole allocated memory for reuse and does not shrink by default since
    /// this map is not designed to ever have a high number of elements and clearing
    /// the underlying vector storage is O(1).
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    ///
    /// let mut v = LinearSet::new();
    /// v.insert(1);
    /// v.clear();
    /// assert!(v.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.map.clear()
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` such that `f(&e)` returns `false`.
    ///
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.map.retain(|k, _| f(k));
    }

    /// Returns `true` if the set contains a value.
    ///
    /// The value may be any borrowed form of the set's value type, but
    /// `Eq` on the borrowed form *must* match those for
    /// the value type.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    ///
    /// let set: LinearSet<_> = [1, 2, 3].iter().cloned().collect();
    /// assert_eq!(set.contains(&1), true);
    /// assert_eq!(set.contains(&4), false);
    /// ```
    pub fn contains<Q: ?Sized>(&self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Eq,
    {
        self.map.contains_key(value)
    }

    /// Returns `true` if the set has no elements in common with `other`.
    /// This is equivalent to checking for an empty intersection.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    ///
    /// let a: LinearSet<_> = [1, 2, 3].iter().cloned().collect();
    /// let mut b = LinearSet::new();
    ///
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(4);
    /// assert_eq!(a.is_disjoint(&b), true);
    /// b.insert(1);
    /// assert_eq!(a.is_disjoint(&b), false);
    /// ```
    pub fn is_disjoint(&self, other: &LinearSet<T>) -> bool {
        self.iter().all(|v| !other.contains(v))
    }

    /// Returns `true` if the set is a subset of another.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    ///
    /// let sup: LinearSet<_> = [1, 2, 3].iter().cloned().collect();
    /// let mut set = LinearSet::new();
    ///
    /// assert_eq!(set.is_subset(&sup), true);
    /// set.insert(2);
    /// assert_eq!(set.is_subset(&sup), true);
    /// set.insert(4);
    /// assert_eq!(set.is_subset(&sup), false);
    /// ```
    pub fn is_subset(&self, other: &LinearSet<T>) -> bool {
        self.iter().all(|v| other.contains(v))
    }

    /// Returns `true` if the set is a superset of another.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    ///
    /// let sub: LinearSet<_> = [1, 2].iter().cloned().collect();
    /// let mut set = LinearSet::new();
    ///
    /// assert_eq!(set.is_superset(&sub), false);
    ///
    /// set.insert(0);
    /// set.insert(1);
    /// assert_eq!(set.is_superset(&sub), false);
    ///
    /// set.insert(2);
    /// assert_eq!(set.is_superset(&sub), true);
    /// ```
    #[inline]
    pub fn is_superset(&self, other: &LinearSet<T>) -> bool {
        other.is_subset(self)
    }

    /// Adds a value to the set.
    ///
    /// If the set did not have a value present, `true` is returned.
    ///
    /// If the set did have this key present, `false` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    ///
    /// let mut set = LinearSet::new();
    ///
    /// assert_eq!(set.insert(2), true);
    /// assert_eq!(set.insert(2), false);
    /// assert_eq!(set.len(), 1);
    /// ```
    pub fn insert(&mut self, value: T) -> bool {
        self.map.insert(value, ()).is_none()
    }

    /// Removes (by swap) a value from the set. Returns `true` if the value was
    /// present in the set.
    ///
    /// The value may be any borrowed form of the set's value type, but
    /// `Eq` on the borrowed form *must* match those for
    /// the value type.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    ///
    /// let mut set = LinearSet::new();
    ///
    /// set.insert(2);
    /// assert_eq!(set.remove(&2), true);
    /// assert_eq!(set.remove(&2), false);
    /// ```
    pub fn remove<Q: ?Sized>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Eq,
    {
        self.map.remove(value).is_some()
    }

    /// Removes a value from the set by shifting all subsequent elements. Returns `true` if the value was
    /// present in the set.
    ///
    /// The value may be any borrowed form of the set's value type, but
    /// `Eq` on the borrowed form *must* match those for
    /// the value type.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    ///
    /// let mut set = LinearSet::new();
    ///
    /// set.insert(2);
    /// assert_eq!(set.remove(&2), true);
    /// assert_eq!(set.remove(&2), false);
    /// ```
    #[inline]
    pub fn shift_remove<Q: ?Sized>(&mut self, value: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Eq,
    {
        self.map.shift_remove(value).is_some()
    }

    /// Sort the map’s key-value pairs by the default ordering of the keys.
    /// 
    /// Since we are guaranteed to have no equal elements, we can use unstable sort by default.
    #[inline]
    pub fn sort_keys(&mut self)
    where
        T: Ord,
    {
        self.map.sort_keys();
    }

    /// Sort the map’s key-value pairs by the given comparison function
    /// 
    /// Since we are guaranteed to have no equal elements, we can use unstable sort by default.
    #[inline]
    pub fn sort_by<F>(&mut self, mut cmp: F)
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        self.map.sort_by(|t1, _, t2, _| cmp(t1, t2))
    }

}

impl<T> PartialEq for LinearSet<T>
where
    T: Eq,
{
    fn eq(&self, other: &LinearSet<T>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().all(|key| other.contains(key))
    }
}

impl<T> Eq for LinearSet<T> where T: Eq {}

impl<T> fmt::Debug for LinearSet<T>
where
    T: Eq + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<T> FromIterator<T> for LinearSet<T>
where
    T: Eq,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> LinearSet<T> {
        let iterator = iter.into_iter();
        let lower = iterator.size_hint().0;
        let mut set = LinearSet::with_capacity(lower);
        set.extend(iterator);
        set
    }
}

impl<T> Extend<T> for LinearSet<T>
where
    T: Eq,
{
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for k in iter {
            self.insert(k);
        }
    }
}

impl<'a, T> Extend<&'a T> for LinearSet<T>
where
    T: 'a + Eq + Copy,
{
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned());
    }
}

impl<T> Default for LinearSet<T>
where
    T: Eq,
{
    fn default() -> LinearSet<T> {
        LinearSet::new()
    }
}

impl<T: Eq> From<LinearSet<T>> for Vec<T> {
    #[inline]
    fn from(other: LinearSet<T>) -> Self {
        let storage = Vec::<_>::from(other.map);
        let mut manually_drop = ManuallyDrop::new(storage);
        // SAFETY:
        // we just made sure that the old vec will not get dropped/freed by putting it in a
        // manually drop.
        //
        // A tuple of (T, ()) is guaranteed to have the same size and alignment as T since `()`
        // is a ZST, so we can directly cast the pointer.
        unsafe {
            Vec::from_raw_parts(
                manually_drop.as_mut_ptr() as *mut (T, ()) as *mut T,
                manually_drop.len(),
                manually_drop.capacity(),
            )
        }
    }
}

impl<T: Eq> From<Vec<T>> for LinearSet<T> {
    #[inline]
    fn from(other: Vec<T>) -> Self {
        let mut manually_drop = ManuallyDrop::new(other);
        // SAFETY:
        // we just made sure that the old vec will not get dropped/freed by putting it in a
        // manually drop.
        //
        // A tuple of (T, ()) is guaranteed to have the same size and alignment as T since `()`
        // is a ZST, so we can directly cast the pointer.
        let storage = unsafe {
            Vec::from_raw_parts(
                manually_drop.as_mut_ptr() as *mut T as *mut (T, ()),
                manually_drop.len(),
                manually_drop.capacity(),
            )
        };

        Self { map: storage.into() }
    }
}

impl<'a, 'b, T> BitOr<&'b LinearSet<T>> for &'a LinearSet<T>
where
    T: Eq + Clone,
{
    type Output = LinearSet<T>;

    /// Returns the union of `self` and `rhs` as a new `LinearSet<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    ///
    /// let a: LinearSet<_> = vec![1, 2, 3].into_iter().collect();
    /// let b: LinearSet<_> = vec![3, 4, 5].into_iter().collect();
    ///
    /// let set = &a | &b;
    ///
    /// let mut i = 0;
    /// let expected = [1, 2, 3, 4, 5];
    /// for x in &set {
    ///     assert!(expected.contains(x));
    ///     i += 1;
    /// }
    /// assert_eq!(i, expected.len());
    /// ```
    fn bitor(self, rhs: &LinearSet<T>) -> LinearSet<T> {
        self.union(rhs).cloned().collect()
    }
}

impl<'a, 'b, T> BitAnd<&'b LinearSet<T>> for &'a LinearSet<T>
where
    T: Eq + Clone,
{
    type Output = LinearSet<T>;

    /// Returns the intersection of `self` and `rhs` as a new `LinearSet<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    ///
    /// let a: LinearSet<_> = vec![1, 2, 3].into_iter().collect();
    /// let b: LinearSet<_> = vec![2, 3, 4].into_iter().collect();
    ///
    /// let set = &a & &b;
    ///
    /// let mut i = 0;
    /// let expected = [2, 3];
    /// for x in &set {
    ///     assert!(expected.contains(x));
    ///     i += 1;
    /// }
    /// assert_eq!(i, expected.len());
    /// ```
    fn bitand(self, rhs: &LinearSet<T>) -> LinearSet<T> {
        self.intersection(rhs).cloned().collect()
    }
}

impl<'a, 'b, T> BitXor<&'b LinearSet<T>> for &'a LinearSet<T>
where
    T: Eq + Clone,
{
    type Output = LinearSet<T>;

    /// Returns the symmetric difference of `self` and `rhs` as a new `LinearSet<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    ///
    /// let a: LinearSet<_> = vec![1, 2, 3].into_iter().collect();
    /// let b: LinearSet<_> = vec![3, 4, 5].into_iter().collect();
    ///
    /// let set = &a ^ &b;
    ///
    /// let mut i = 0;
    /// let expected = [1, 2, 4, 5];
    /// for x in &set {
    ///     assert!(expected.contains(x));
    ///     i += 1;
    /// }
    /// assert_eq!(i, expected.len());
    /// ```
    fn bitxor(self, rhs: &LinearSet<T>) -> LinearSet<T> {
        self.symmetric_difference(rhs).cloned().collect()
    }
}

impl<'a, 'b, T> Sub<&'b LinearSet<T>> for &'a LinearSet<T>
where
    T: Eq + Clone,
{
    type Output = LinearSet<T>;

    /// Returns the difference of `self` and `rhs` as a new `LinearSet<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    ///
    /// let a: LinearSet<_> = vec![1, 2, 3].into_iter().collect();
    /// let b: LinearSet<_> = vec![3, 4, 5].into_iter().collect();
    ///
    /// let set = &a - &b;
    ///
    /// let mut i = 0;
    /// let expected = [1, 2];
    /// for x in &set {
    ///     assert!(expected.contains(x));
    ///     i += 1;
    /// }
    /// assert_eq!(i, expected.len());
    /// ```
    fn sub(self, rhs: &LinearSet<T>) -> LinearSet<T> {
        self.difference(rhs).cloned().collect()
    }
}

/// [`LinearSet`] iterator
#[allow(missing_debug_implementations)]
pub struct Iter<'a, K> {
    iter: Keys<'a, K, ()>,
}

/// [`LinearSet`] move iterator
#[allow(missing_debug_implementations)]
pub struct IntoIter<K> {
    iter: super::linear_map::IntoIter<K, ()>,
}

/// [`LinearSet`] drain iterator
#[allow(missing_debug_implementations)]
pub struct Drain<'a, K> {
    iter: super::linear_map::Drain<'a, K, ()>,
}

/// Intersection iterator
#[allow(missing_debug_implementations)]
pub struct Intersection<'a, T> {
    // iterator of the first set
    iter: Iter<'a, T>,
    // the second set
    other: &'a LinearSet<T>,
}

/// Difference iterator
#[allow(missing_debug_implementations)]
pub struct Difference<'a, T> {
    // iterator of the first set
    iter: Iter<'a, T>,
    // the second set
    other: &'a LinearSet<T>,
}

/// Symmetric difference iterator.
#[allow(missing_debug_implementations)]
pub struct SymmetricDifference<'a, T> {
    iter: Chain<Difference<'a, T>, Difference<'a, T>>,
}

/// Set union iterator.
#[allow(missing_debug_implementations)]
pub struct Union<'a, T> {
    iter: Chain<Iter<'a, T>, Difference<'a, T>>,
}

impl<'a, T> IntoIterator for &'a LinearSet<T>
where
    T: Eq,
{
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

impl<T> IntoIterator for LinearSet<T>
where
    T: Eq,
{
    type Item = T;
    type IntoIter = IntoIter<T>;

    /// Creates a consuming iterator, that is, one that moves each value out
    /// of the set. The set cannot be used after calling this.
    ///
    /// # Examples
    ///
    /// ```
    /// use tame_containers::linear_set::LinearSet;;
    /// let mut set = LinearSet::new();
    /// set.insert("a".to_string());
    /// set.insert("b".to_string());
    ///
    /// // Not possible to collect to a Vec<String> with a regular `.iter()`.
    /// let v: Vec<String> = set.into_iter().collect();
    ///
    /// for x in &v {
    ///     println!("{}", x);
    /// }
    /// ```
    fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            iter: self.map.into_iter(),
        }
    }
}

impl<'a, K> Clone for Iter<'a, K> {
    fn clone(&self) -> Iter<'a, K> {
        Iter {
            iter: self.iter.clone(),
        }
    }
}
impl<'a, K> Iterator for Iter<'a, K> {
    type Item = &'a K;

    fn next(&mut self) -> Option<&'a K> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
impl<'a, K> ExactSizeIterator for Iter<'a, K> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<K> Iterator for IntoIter<K> {
    type Item = K;

    fn next(&mut self) -> Option<K> {
        self.iter.next().map(|(k, _)| k)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
impl<K> ExactSizeIterator for IntoIter<K> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, K> Iterator for Drain<'a, K> {
    type Item = K;

    fn next(&mut self) -> Option<K> {
        self.iter.next().map(|(k, _)| k)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
impl<'a, K> ExactSizeIterator for Drain<'a, K> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<'a, T> Clone for Intersection<'a, T> {
    fn clone(&self) -> Intersection<'a, T> {
        Intersection {
            iter: self.iter.clone(),
            ..*self
        }
    }
}

impl<'a, T> Iterator for Intersection<'a, T>
where
    T: Eq,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        loop {
            match self.iter.next() {
                None => return None,
                Some(elt) => {
                    if self.other.contains(elt) {
                        return Some(elt);
                    }
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }
}

impl<'a, T> Clone for Difference<'a, T> {
    fn clone(&self) -> Difference<'a, T> {
        Difference {
            iter: self.iter.clone(),
            ..*self
        }
    }
}

impl<'a, T> Iterator for Difference<'a, T>
where
    T: Eq,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        loop {
            match self.iter.next() {
                None => return None,
                Some(elt) => {
                    if !self.other.contains(elt) {
                        return Some(elt);
                    }
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, upper) = self.iter.size_hint();
        (0, upper)
    }
}

impl<'a, T> Clone for SymmetricDifference<'a, T> {
    fn clone(&self) -> SymmetricDifference<'a, T> {
        SymmetricDifference {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, T> Iterator for SymmetricDifference<'a, T>
where
    T: Eq,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, T> Clone for Union<'a, T> {
    fn clone(&self) -> Union<'a, T> {
        Union {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, T> Iterator for Union<'a, T>
where
    T: Eq,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        self.iter.next()
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[allow(dead_code)]
fn assert_covariance() {
    fn set<'new>(v: LinearSet<&'static str>) -> LinearSet<&'new str> {
        v
    }
    fn iter<'a, 'new>(v: Iter<'a, &'static str>) -> Iter<'a, &'new str> {
        v
    }
    fn into_iter<'new>(v: IntoIter<&'static str>) -> IntoIter<&'new str> {
        v
    }
    fn difference<'a, 'new>(v: Difference<'a, &'static str>) -> Difference<'a, &'new str> {
        v
    }
    fn symmetric_difference<'a, 'new>(
        v: SymmetricDifference<'a, &'static str>,
    ) -> SymmetricDifference<'a, &'new str> {
        v
    }
    fn intersection<'a, 'new>(v: Intersection<'a, &'static str>) -> Intersection<'a, &'new str> {
        v
    }
    fn union<'a, 'new>(v: Union<'a, &'static str>) -> Union<'a, &'new str> {
        v
    }
}