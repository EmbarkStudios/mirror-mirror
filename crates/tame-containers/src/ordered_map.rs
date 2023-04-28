use indexmap::Equivalent;

use core::cmp::Ordering;
use core::fmt;
use core::hash::BuildHasher;
use core::hash::Hash;
use core::hash::Hasher;
use core::ops::RangeBounds;

use indexmap::IndexMap;

pub use indexmap::map::Drain;
pub use indexmap::map::Entry;
pub use indexmap::map::IntoIter;
pub use indexmap::map::IntoKeys;
pub use indexmap::map::IntoValues;
pub use indexmap::map::Iter;
pub use indexmap::map::IterMut;
pub use indexmap::map::Keys;
pub use indexmap::map::Values;
pub use indexmap::map::ValuesMut;

/// A key-to-value map that has a specified order of contained elements.
///
/// The order is *not* automatically maintained, thus you can move element order as you please, or sort
/// with the various sorting functions.
///
/// This is a wrapper around [`indexmap::IndexMap`] which implements various traits in ways that fit
/// our use cases better than the choices `indexmap` made. If you really need to access the wrapped map directly,
/// you can do so with the `inner`, `inner_mut` or `into_inner` methods, but be careful as the semantics of the traits
/// mentioned below may be different.
///
/// Implements `PartialEq`, `Eq`, and `Hash` such that two maps are equal and hash to the same value if they have
/// the same `(k, v)` element pairs ***and*** the same order of those elements.
///
/// Implements `Ord` with [lexographical] ordering between element pairs.
///
/// Implements `serde::Serialize` and `serde::Deserialize` by serializing as a sequence of `(k, v)` pairs, rather
/// than as a native map object. This is for better compatibility with JSON, which only allows strings as key for
/// native JSON maps.
///
/// [lexographical]: core::cmp::Ord#lexographical-comparison
pub struct OrderedMap<K, V, S = ahash::RandomState> {
    pub(crate) inner: IndexMap<K, V, S>,
}

impl<K, V> OrderedMap<K, V, ahash::RandomState> {
    /// [`IndexMap::new`] but using an [`ahash`] hasher.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// [`IndexMap::with_capacity`] but using an [`ahash`] hasher.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, ahash::RandomState::default())
    }
}

impl<K, V, S> OrderedMap<K, V, S> {
    /// See [`IndexMap::with_capacity_and_hasher`]
    #[inline]
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        Self {
            inner: IndexMap::<K, V, S>::with_capacity_and_hasher(capacity, hasher),
        }
    }

    /// See [`IndexMap::len`]
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// See [`IndexMap::capacity`]
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// See [`IndexMap::keys`]
    #[inline]
    pub fn keys(&self) -> Keys<'_, K, V> {
        self.inner.keys()
    }

    /// See [`IndexMap::into_keys`]
    #[inline]
    pub fn into_keys(self) -> IntoKeys<K, V> {
        self.inner.into_keys()
    }

    /// See [`IndexMap::values`]
    #[inline]
    pub fn values(&self) -> Values<'_, K, V> {
        self.inner.values()
    }

    /// See [`IndexMap::values_mut`]
    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        self.inner.values_mut()
    }

    /// See [`IndexMap::into_values`]
    #[inline]
    pub fn into_values(self) -> IntoValues<K, V> {
        self.inner.into_values()
    }

    /// See [`IndexMap::iter`]
    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        self.inner.iter()
    }

    /// See [`IndexMap::iter_mut`]
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        self.inner.iter_mut()
    }

    /// See [`IndexMap::is_empty`]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// See [`IndexMap::drain`]
    #[inline]
    pub fn drain<R>(&mut self, range: R) -> Drain<'_, K, V>
    where
        R: RangeBounds<usize>,
    {
        self.inner.drain(range)
    }

    /// See [`IndexMap::clear`]
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    /// See [`IndexMap::hasher`]
    #[inline]
    pub fn hasher(&self) -> &S {
        self.inner.hasher()
    }

    /// Access the wrapped [`IndexMap`].
    #[inline]
    pub fn inner(&self) -> &IndexMap<K, V, S> {
        &self.inner
    }

    /// Access the wrapped [`IndexMap`] mutably.
    #[inline]
    pub fn inner_mut(&mut self) -> &mut IndexMap<K, V, S> {
        &mut self.inner
    }

    /// Extract the wrapped [`IndexMap`].
    #[inline]
    pub fn into_inner(self) -> IndexMap<K, V, S> {
        self.inner
    }
}

impl<K, V, const N: usize> From<[(K, V); N]> for OrderedMap<K, V, ahash::RandomState>
where
    K: Hash + Eq,
{
    fn from(arr: [(K, V); N]) -> Self {
        Self {
            inner: IndexMap::<K, V, ahash::RandomState>::from_iter(arr),
        }
    }
}

impl<K, V, S> OrderedMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    /// See [`IndexMap::retain`]
    #[inline]
    pub fn retain<F>(&mut self, keep: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        self.inner.retain(keep)
    }

    /// See [`IndexMap::reserve`]
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional)
    }

    /// See [`IndexMap::entry`]
    #[inline]
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        self.inner.entry(key)
    }

    /// See [`IndexMap::get`]
    #[inline]
    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
    where
        Q: Hash + Equivalent<K>,
    {
        self.inner.get(k)
    }

    /// See [`IndexMap::get_key_value`]
    #[inline]
    pub fn get_key_value<Q: ?Sized>(&self, k: &Q) -> Option<(&K, &V)>
    where
        Q: Hash + Equivalent<K>,
    {
        self.inner.get_key_value(k)
    }

    /// See [`IndexMap::get_full]
    pub fn get_full<Q: ?Sized>(&self, key: &Q) -> Option<(usize, &K, &V)>
    where
        Q: Hash + Equivalent<K>,
    {
        self.inner.get_full(key)
    }

    /// See [`IndexMap::get_mut`]
    #[inline]
    pub fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        Q: Hash + Equivalent<K>,
    {
        self.inner.get_mut(k)
    }

    /// See [`IndexMap::get_full`]
    pub fn get_full_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<(usize, &K, &mut V)>
    where
        Q: Hash + Equivalent<K>,
    {
        self.inner.get_full_mut(key)
    }
    /// See [`IndexMap::contains_key`]
    #[inline]
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        Q: Hash + Equivalent<K>,
    {
        self.inner.contains_key(key)
    }

    /// See [`IndexMap::get_index_of`]
    #[inline]
    pub fn get_index_of<Q: ?Sized>(&self, key: &Q) -> Option<usize>
    where
        Q: Hash + Equivalent<K>,
    {
        self.inner.get_index_of(key)
    }

    /// See [`IndexMap::insert`]
    #[inline]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.inner.insert(k, v)
    }

    /// See [`IndexMap::insert_full`]
    #[inline]
    pub fn insert_full(&mut self, key: K, value: V) -> (usize, Option<V>) {
        self.inner.insert_full(key, value)
    }

    /// See [`IndexMap::remove`]
    ///
    /// **NOTE:** This is equivalent to `.swap_remove(key)`, if you need to
    /// preserve the order of the keys in the map, use `.shift_remove_entry(key)`
    /// instead.
    #[inline]
    pub fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<V>
    where
        Q: Hash + Equivalent<K>,
    {
        self.inner.remove(k)
    }

    /// See [`IndexMap::remove_entry`]
    ///
    /// **NOTE:** This is equivalent to `.swap_remove_entry(key)`, if you need to
    /// preserve the order of the keys in the map, use `.shift_remove_entry(key)`
    /// instead.
    #[inline]
    pub fn remove_entry<Q: ?Sized>(&mut self, key: &Q) -> Option<(K, V)>
    where
        Q: Hash + Equivalent<K>,
    {
        self.inner.remove_entry(key)
    }

    /// See [`IndexMap::swap_remove`]
    #[inline]
    pub fn swap_remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        Q: Hash + Equivalent<K>,
    {
        self.inner.swap_remove(key)
    }

    /// See [`IndexMap::swap_remove_entry`]
    #[inline]
    pub fn swap_remove_entry<Q: ?Sized>(&mut self, key: &Q) -> Option<(K, V)>
    where
        Q: Hash + Equivalent<K>,
    {
        self.inner.swap_remove_entry(key)
    }

    /// See [`IndexMap::swap_remove_full`]
    #[inline]
    pub fn swap_remove_full<Q: ?Sized>(&mut self, key: &Q) -> Option<(usize, K, V)>
    where
        Q: Hash + Equivalent<K>,
    {
        self.inner.swap_remove_full(key)
    }

    /// See [`IndexMap::shift_remove`]
    #[inline]
    pub fn shift_remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        Q: Hash + Equivalent<K>,
    {
        self.inner.shift_remove(key)
    }

    /// See [`IndexMap::shift_remove_entry`]
    #[inline]
    pub fn shift_remove_entry<Q: ?Sized>(&mut self, key: &Q) -> Option<(K, V)>
    where
        Q: Hash + Equivalent<K>,
    {
        self.inner.shift_remove_entry(key)
    }

    /// See [`IndexMap::shift_remove_full`]
    #[inline]
    pub fn shift_remove_full<Q: ?Sized>(&mut self, key: &Q) -> Option<(usize, K, V)>
    where
        Q: Hash + Equivalent<K>,
    {
        self.inner.shift_remove_full(key)
    }

    /// See [`IndexMap::pop`]
    #[inline]
    pub fn pop(&mut self) -> Option<(K, V)> {
        self.inner.pop()
    }

    /// See [`IndexMap::sort_keys`]
    #[inline]
    pub fn sort_keys(&mut self)
    where
        K: Ord,
    {
        self.inner.sort_keys()
    }

    /// See [`IndexMap::sort_by`]
    #[inline]
    pub fn sort_by<F>(&mut self, cmp: F)
    where
        F: FnMut(&K, &V, &K, &V) -> Ordering,
    {
        self.inner.sort_by(cmp)
    }

    /// See [`IndexMap::sorted_by`]
    #[inline]
    pub fn sorted_by<F>(self, cmp: F) -> IntoIter<K, V>
    where
        F: FnMut(&K, &V, &K, &V) -> Ordering,
    {
        self.inner.sorted_by(cmp)
    }

    /// See [`IndexMap::sort_unstable_keys`]
    #[inline]
    pub fn sort_unstable_keys(&mut self)
    where
        K: Ord,
    {
        self.inner.sort_unstable_keys()
    }

    /// See [`IndexMap::sort_unstable_by`]
    #[inline]
    pub fn sort_unstable_by<F>(&mut self, cmp: F)
    where
        F: FnMut(&K, &V, &K, &V) -> Ordering,
    {
        self.inner.sort_unstable_by(cmp)
    }

    /// See [`IndexMap::sorted_unstable_by`]
    #[inline]
    pub fn sorted_unstable_by<F>(self, cmp: F) -> IntoIter<K, V>
    where
        F: FnMut(&K, &V, &K, &V) -> Ordering,
    {
        self.inner.sorted_unstable_by(cmp)
    }

    /// See [`IndexMap::reverse`]
    #[inline]
    pub fn reverse(&mut self) {
        self.inner.reverse()
    }
}

impl<K, V, S> OrderedMap<K, V, S> {
    /// See [`IndexMap::get_index`]
    #[inline]
    pub fn get_index(&self, index: usize) -> Option<(&K, &V)> {
        self.inner.get_index(index)
    }

    /// See [`IndexMap::get_index_mut`]
    #[inline]
    pub fn get_index_mut(&mut self, index: usize) -> Option<(&mut K, &mut V)> {
        self.inner.get_index_mut(index)
    }

    /// See [`IndexMap::first`]
    #[inline]
    pub fn first(&self) -> Option<(&K, &V)> {
        self.inner.first()
    }

    /// See [`IndexMap::first_mut`]
    #[inline]
    pub fn first_mut(&mut self) -> Option<(&K, &mut V)> {
        self.inner.first_mut()
    }

    /// See [`IndexMap::last`]
    #[inline]
    pub fn last(&self) -> Option<(&K, &V)> {
        self.inner.last()
    }

    /// See [`IndexMap::last_mut`]
    #[inline]
    pub fn last_mut(&mut self) -> Option<(&K, &mut V)> {
        self.inner.last_mut()
    }

    /// See [`IndexMap::swap_remove_index`]
    #[inline]
    pub fn swap_remove_index(&mut self, index: usize) -> Option<(K, V)> {
        self.inner.swap_remove_index(index)
    }

    /// See [`IndexMap::shift_remove_index`]
    #[inline]
    pub fn shift_remove_index(&mut self, index: usize) -> Option<(K, V)> {
        self.inner.shift_remove_index(index)
    }

    /// See [`IndexMap::move_index`]
    #[inline]
    pub fn move_index(&mut self, from: usize, to: usize) {
        self.inner.move_index(from, to)
    }

    /// See [`IndexMap::swap_indices`]
    #[inline]
    pub fn swap_indices(&mut self, a: usize, b: usize) {
        self.inner.swap_indices(a, b)
    }
}

impl<K, V, S> Clone for OrderedMap<K, V, S>
where
    K: Clone,
    V: Clone,
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

impl<K, V, S> Default for OrderedMap<K, V, S>
where
    S: BuildHasher + Default,
{
    #[inline]
    fn default() -> Self {
        Self {
            inner: IndexMap::with_hasher(S::default()),
        }
    }
}

impl<K, V, S> fmt::Debug for OrderedMap<K, V, S>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OrderedMap")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<K, V, S> PartialEq for OrderedMap<K, V, S>
where
    K: Eq + Hash,
    V: PartialEq,
    S: BuildHasher,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        // lexographical equality, meaning all elements must be in the same order and be equal.
        // short circuit as soon as there's disagreement.
        self.inner
            .iter()
            .zip(other.inner.iter())
            .all(|(self_elt, other_elt)| self_elt == other_elt)
    }
}

impl<K, V, S> Eq for OrderedMap<K, V, S>
where
    K: Eq + Hash,
    V: Eq,
    S: BuildHasher,
{
}

impl<K, V, S> PartialOrd for OrderedMap<K, V, S>
where
    K: Eq + Hash + Ord,
    V: Ord,
    S: BuildHasher,
{
    #[inline]
    fn partial_cmp(&self, other: &OrderedMap<K, V, S>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K, V, S> Ord for OrderedMap<K, V, S>
where
    K: Eq + Hash + Ord,
    V: Ord,
    S: BuildHasher,
{
    fn cmp(&self, other: &OrderedMap<K, V, S>) -> Ordering {
        // first compare lengths, if equal, we do lexographical ordering...
        match self.len().cmp(&other.len()) {
            Ordering::Less => return Ordering::Less,
            Ordering::Greater => return Ordering::Greater,
            Ordering::Equal => (),
        }
        self.iter().cmp(other.iter())
    }
}

impl<K, V, S> Hash for OrderedMap<K, V, S>
where
    K: Hash,
    V: Hash,
    S: BuildHasher,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        for elt in self.inner.iter() {
            elt.hash(state);
        }
    }
}

impl<K, V, S> FromIterator<(K, V)> for OrderedMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher + Default,
{
    #[inline]
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self {
            inner: IndexMap::<K, V, S>::from_iter(iter),
        }
    }
}

impl<K, V, S> IntoIterator for OrderedMap<K, V, S> {
    type Item = (K, V);
    type IntoIter = indexmap::map::IntoIter<K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a OrderedMap<K, V, S> {
    type Item = (&'a K, &'a V);
    type IntoIter = indexmap::map::Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut OrderedMap<K, V, S> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = indexmap::map::IterMut<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}

#[cfg(feature = "serde")]
impl<K, V, RS> serde::Serialize for OrderedMap<K, V, RS>
where
    K: serde::Serialize,
    V: serde::Serialize,
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
impl<'de, K, V, S> serde::Deserialize<'de> for OrderedMap<K, V, S>
where
    K: serde::Deserialize<'de> + Eq + Hash,
    V: serde::Deserialize<'de>,
    S: BuildHasher + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use core::marker::PhantomData;
        use serde::de::SeqAccess;
        use serde::de::Visitor;

        struct IndexMapSeqVisitor<K, V, S>(PhantomData<(K, V, S)>);

        impl<'de, K, V, S> Visitor<'de> for IndexMapSeqVisitor<K, V, S>
        where
            K: serde::Deserialize<'de> + Eq + Hash,
            V: serde::Deserialize<'de>,
            S: BuildHasher + Default,
        {
            type Value = IndexMap<K, V, S>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of (k, v) pairs")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut map =
                    IndexMap::with_capacity_and_hasher(seq.size_hint().unwrap_or(0), S::default());

                while let Some((k, v)) = seq.next_element::<(K, V)>()? {
                    map.insert(k, v);
                }

                Ok(map)
            }
        }

        let map = deserializer.deserialize_seq(IndexMapSeqVisitor::<K, V, S>(PhantomData))?;
        Ok(Self { inner: map })
    }
}

#[cfg(feature = "speedy")]
impl<'a, C, K, V, S> speedy::Readable<'a, C> for OrderedMap<K, V, S>
where
    C: speedy::Context,
    K: speedy::Readable<'a, C> + Eq + Hash,
    V: speedy::Readable<'a, C>,
    S: BuildHasher + Default,
{
    fn read_from<R: speedy::Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let length = reader.read_u32()? as usize;
        let map = (0..length)
            .map(|_| -> Result<_, <C as speedy::Context>::Error> {
                let key = K::read_from(reader)?;
                let value = V::read_from(reader)?;
                Ok((key, value))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self { inner: map })
    }
}

#[cfg(feature = "speedy")]
impl<C, K, V, S> speedy::Writable<C> for OrderedMap<K, V, S>
where
    C: speedy::Context,
    K: speedy::Writable<C>,
    V: speedy::Writable<C>,
{
    fn write_to<T: ?Sized + speedy::Writer<C>>(
        &self,
        writer: &mut T,
    ) -> Result<(), <C as speedy::Context>::Error> {
        writer.write_u32(self.inner.len() as u32)?;
        for (key, value) in &self.inner {
            key.write_to(writer)?;
            value.write_to(writer)?;
        }
        Ok(())
    }
}
