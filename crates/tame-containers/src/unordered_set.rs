use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt;
use core::hash::BuildHasher;
use core::hash::Hash;
use core::hash::Hasher;

use std::collections::HashSet;

pub use std::collections::hash_set::Drain;
pub use std::collections::hash_set::IntoIter;
pub use std::collections::hash_set::Iter;

use crate::STATIC_RANDOM_STATE;

/// A deduplicated set that does not have a specified order of contained elements.
///
/// It is a good choice to use this set if you plan to do insertion, removal, and lookup significantly
/// more often than iteration of the contained elements. If you will iterate the elements often (even if you don't
/// specifically care about their order), think about using an [`OrderedSet`] instead.
///
/// This is a wrapper around [`std::collections::HashSet`] which implements various traits in ways that fit
/// our use cases better than the choices `std` made. If you really need to access the wrapped [`HashSet`],
/// you can do so with the `inner`, `inner_mut`, and `into_inner` methods. However, be careful as using these
/// has different trait implementation semantics as mentioned below.
///
/// Implements `PartialEq`, `Eq`, and `Hash` such that two sets are equal and hash to the same value if they have
/// the same elements, regardless of order. However, the `Hash` implementation is not fully cryptographically secure.
///
/// Implements `Ord`, *but* the implementation requires that for sets
/// of the same length, we allocate a `Vec` containing all the elements for both sets,
/// sort them, and then do [lexographical] ordering between them,
/// which is very slow and it's not recommended to use this functionality if at all possible.
///
/// [`OrderedSet`]: crate::OrderedSet
/// [lexographical]: core::cmp::Ord#lexographical-comparison
pub struct UnorderedSet<T, S = ahash::RandomState> {
    pub(crate) inner: HashSet<T, S>,
}

impl<T> UnorderedSet<T, ahash::RandomState> {
    /// [`HashSet::new`] but using an [`ahash`] hasher.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// [`HashSet::with_capacity`] but using an [`ahash`] hasher.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, ahash::RandomState::default())
    }
}

impl<T, S> UnorderedSet<T, S> {
    /// See [`HashSet::with_capacity_and_hasher`]
    #[inline]
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        Self {
            inner: HashSet::<T, S>::with_capacity_and_hasher(capacity, hasher),
        }
    }

    /// See [`HashSet::len`]
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// See [`HashSet::capacity`]
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// See [`HashSet::iter`]
    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        self.inner.iter()
    }

    /// See [`HashSet::is_empty`]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// See [`HashSet::drain`]
    #[inline]
    pub fn drain(&mut self) -> Drain<'_, T> {
        self.inner.drain()
    }

    /// See [`HashSet::retain`]
    #[inline]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.inner.retain(f)
    }

    /// See [`HashSet::clear`]
    ///
    /// Note that this method does not shrink the underlying allocation (keeps capacity the same) and is `O(capacity)`.
    /// Thus repeated calls to `clear_no_shrink` on a set that is far under-occupied may be unexpectedly expensive. Consider using
    /// [`clear_and_shrink`] or [`clear_and_shrink_to`] to shrink the underlying allocation when appropriate when clearing.
    ///
    /// [`clear_and_shrink`]: UnorderedSet::clear_and_shrink
    /// [`clear_and_shrink_to`]: UnorderedSet::clear_and_shrink_to
    #[inline]
    pub fn clear_no_shrink(&mut self) {
        self.inner.clear()
    }

    /// See [`HashSet::hasher`]
    #[inline]
    pub fn hasher(&self) -> &S {
        self.inner.hasher()
    }

    /// Access the wrapped [`HashSet`].
    #[inline]
    pub fn inner(&self) -> &HashSet<T, S> {
        &self.inner
    }

    /// Access the wrapped [`HashSet`] mutably.
    #[inline]
    pub fn inner_mut(&mut self) -> &mut HashSet<T, S> {
        &mut self.inner
    }

    /// Extract the wrapped [`HashSet`].
    #[inline]
    pub fn into_inner(self) -> HashSet<T, S> {
        self.inner
    }
}

impl<T, const N: usize> From<[T; N]> for UnorderedSet<T, ahash::RandomState>
where
    T: Hash + Eq,
{
    fn from(arr: [T; N]) -> Self {
        Self {
            inner: HashSet::<T, ahash::RandomState>::from_iter(arr),
        }
    }
}

impl<T, S> UnorderedSet<T, S>
where
    T: Hash + Eq,
    S: BuildHasher,
{
    /// See [`HashSet::reserve`]
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional)
    }

    /// See [`HashSet::try_reserve`]
    #[inline]
    pub fn try_reserve(
        &mut self,
        additional: usize,
    ) -> Result<(), std::collections::TryReserveError> {
        self.inner.try_reserve(additional)
    }

    /// See [`HashSet::shrink_to_fit`]
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit();
    }

    /// See [`HashSet::shrink_to`]
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.inner.shrink_to(min_capacity);
    }

    /// Clears the set, removing all elements.
    ///
    /// Note that this shrinks the capacity of the set based on a basic heuristic. See [`clear_and_shrink`] for more details, which this
    /// method redirects to internally.
    ///
    /// [`clear_and_shrink`]: UnorderedSet::clear_and_shrink
    #[inline]
    pub fn clear(&mut self) {
        self.clear_and_shrink()
    }

    /// Clears and shrinks the capacity of the set on a basic heuristic. If you have a more specific heuristic, see [`clear_and_shrink_to`].
    ///
    /// If the set previously had > 128 element capacity, shrinks to whichever is larger between 128 and 110% of the previous length of the set
    /// in an effort to reduce reallocation for repeated use-and-clear on similar numbers of items. If the set had <= 128 element capacity, no shrink happens.
    ///
    /// [`clear_and_shrink_to`]: UnorderedSet::clear_and_shrink_to
    #[inline]
    pub fn clear_and_shrink(&mut self) {
        if self.capacity() > 128 {
            let new_cap = 128usize.max((self.len() as f64 * 1.1) as usize);
            self.clear_and_shrink_to(new_cap);
        } else {
            self.clear_no_shrink();
        }
    }

    /// Clears and shrinks the capacity of the set to the given capacity.
    #[inline]
    pub fn clear_and_shrink_to(&mut self, capacity: usize) {
        self.clear_no_shrink();
        self.shrink_to(capacity);
    }

    /// See [`HashSet::get`]
    #[inline]
    pub fn get<Q: ?Sized>(&self, elt: &Q) -> Option<&T>
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.inner.get(elt)
    }

    /// See [`HashSet::insert`]
    #[inline]
    pub fn insert(&mut self, elt: T) -> bool {
        self.inner.insert(elt)
    }

    /// See [`HashSet::remove`]
    #[inline]
    pub fn remove<Q: ?Sized>(&mut self, elt: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.inner.remove(elt)
    }

    /// See [`HashSet::contains`]
    #[inline]
    pub fn contains<Q: ?Sized>(&self, elt: &Q) -> bool
    where
        T: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.inner.contains(elt)
    }
}

impl<T, S> Clone for UnorderedSet<T, S>
where
    T: Clone,
    S: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }

    #[inline]
    fn clone_from(&mut self, other: &Self) {
        self.inner.clone_from(&other.inner);
    }
}

impl<T, S> Default for UnorderedSet<T, S>
where
    S: BuildHasher + Default,
{
    #[inline]
    fn default() -> Self {
        Self {
            inner: HashSet::with_hasher(S::default()),
        }
    }
}

impl<T, S> fmt::Debug for UnorderedSet<T, S>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnorderedSet")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T, S> PartialEq for UnorderedSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<T, S> Eq for UnorderedSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
}

impl<T, S> PartialOrd for UnorderedSet<T, S>
where
    T: Eq + Hash + Ord,
    S: BuildHasher,
{
    #[inline]
    fn partial_cmp(&self, other: &UnorderedSet<T, S>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T, S> Ord for UnorderedSet<T, S>
where
    T: Eq + Hash + Ord,
    S: BuildHasher,
{
    fn cmp(&self, other: &UnorderedSet<T, S>) -> Ordering {
        // first compare lengths, if equal, we have to sort and do lexographical ordering...
        match self.len().cmp(&other.len()) {
            Ordering::Less => return Ordering::Less,
            Ordering::Greater => return Ordering::Greater,
            Ordering::Equal => (),
        }
        let mut self_seq = self.inner.iter().collect::<Vec<_>>();
        self_seq.sort();
        let mut other_seq = other.inner.iter().collect::<Vec<_>>();
        other_seq.sort();
        self_seq.into_iter().cmp(other_seq)
    }
}

impl<T, S> Hash for UnorderedSet<T, S>
where
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        // although this is in theory not fully cryptographically secure, we don't care about that here.
        // also, since we are using a deterministic hasher, we don't have to worry about differences between sets' hashers.
        // Thus we can use xor as an order-independent hash-combination function. Thus, this
        // will create the same hash as long as two graphs have the same elements, regardless of order.
        // this also satisfies the requirement that Eq and Hash have the same semantics, as HashSet's PartialEq and Eq
        // also have this semantics that as long as the elements are equal the sets are equal.
        let mut hash = 0u64;
        for elt in self.inner.iter() {
            let elt_hash = STATIC_RANDOM_STATE.hash_one(elt);
            hash ^= elt_hash;
        }
        state.write_u64(hash);
    }
}

impl<T, S> FromIterator<T> for UnorderedSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher + Default,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self {
            inner: HashSet::<T, S>::from_iter(iter),
        }
    }
}

impl<T, S> IntoIterator for UnorderedSet<T, S> {
    type Item = T;
    type IntoIter = std::collections::hash_set::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a, T, S> IntoIterator for &'a UnorderedSet<T, S> {
    type Item = &'a T;
    type IntoIter = std::collections::hash_set::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

#[cfg(feature = "serde")]
impl<T, RS> serde::Serialize for UnorderedSet<T, RS>
where
    T: serde::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.inner.len()))?;
        for elt in self.inner.iter() {
            seq.serialize_element(&elt)?;
        }
        seq.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, T, S> serde::Deserialize<'de> for UnorderedSet<T, S>
where
    T: serde::Deserialize<'de> + Eq + Hash,
    S: BuildHasher + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use core::marker::PhantomData;
        use serde::de::SeqAccess;
        use serde::de::Visitor;

        struct HashSetSeqVisitor<T, S>(PhantomData<(T, S)>);

        impl<'de, T, S> Visitor<'de> for HashSetSeqVisitor<T, S>
        where
            T: serde::Deserialize<'de> + Eq + Hash,
            S: BuildHasher + Default,
        {
            type Value = HashSet<T, S>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of T")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut set =
                    HashSet::with_capacity_and_hasher(seq.size_hint().unwrap_or(0), S::default());

                while let Some(elt) = seq.next_element::<T>()? {
                    set.insert(elt);
                }

                Ok(set)
            }
        }

        let set = deserializer.deserialize_seq(HashSetSeqVisitor::<T, S>(PhantomData))?;
        Ok(Self { inner: set })
    }
}

#[cfg(feature = "speedy")]
impl<'a, C, T, S> speedy::Readable<'a, C> for UnorderedSet<T, S>
where
    C: speedy::Context,
    T: speedy::Readable<'a, C> + Eq + Hash,
    S: BuildHasher + Default,
{
    fn read_from<R: speedy::Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let set = HashSet::<T, S>::read_from(reader)?;
        Ok(Self { inner: set })
    }
}

#[cfg(feature = "speedy")]
impl<C, T, S> speedy::Writable<C> for UnorderedSet<T, S>
where
    C: speedy::Context,
    T: speedy::Writable<C>,
{
    fn write_to<W: ?Sized + speedy::Writer<C>>(
        &self,
        writer: &mut W,
    ) -> Result<(), <C as speedy::Context>::Error> {
        HashSet::<T, S>::write_to(&self.inner, writer)
    }
}
