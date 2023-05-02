use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt;
use core::hash::BuildHasher;
use core::hash::Hash;
use core::hash::Hasher;

use std::collections::HashMap;

pub use std::collections::hash_map::Drain;
pub use std::collections::hash_map::Entry;
pub use std::collections::hash_map::IntoIter;
pub use std::collections::hash_map::IntoKeys;
pub use std::collections::hash_map::IntoValues;
pub use std::collections::hash_map::Iter;
pub use std::collections::hash_map::IterMut;
pub use std::collections::hash_map::Keys;
pub use std::collections::hash_map::Values;
pub use std::collections::hash_map::ValuesMut;

use crate::STATIC_RANDOM_STATE;

/// A key-to-value map that does not have a specified order of contained elements.
///
/// It is a good choice to use this map if you plan to do insertion, removal, and lookup by key significantly
/// more often than iteration of the contained elements. If you will iterate the elements often (even if you don't
/// specifically care about their order), think about using an [`OrderedMap`] instead.
///
/// This is a wrapper around [`std::collections::HashMap`] which implements various traits in ways that fit
/// our use cases better than the choices `std` made. If you really need to access the wrapped [`HashMap`],
/// you can do so with the `inner`, `inner_mut`, and `into_inner` methods. However, be careful as using these
/// has different trait implementation semantics as mentioned below.
///
/// Implements `PartialEq`, `Eq`, and `Hash` such that two maps are equal and hash to the same value if they have
/// the same `(k, v)` element pairs. However, the `Hash` implementation is not fully cryptographically secure.
///
/// Implements `Ord`, *but* the implementation requires that for maps
/// of the same length, we allocate a `Vec` containing all the `(k, v)` element pairs for both maps,
/// sort them by `k`, and then do [lexographical] ordering between them,
/// which is very slow and it's not recommended to use this functionality if at all possible.
///
/// Implements `serde::Serialize` and `serde::Deserialize` by serializing as a sequence of `(k, v)` pairs, rather
/// than as a native map object. This is for better compatibility with JSON, which only allows strings as key for
/// native JSON maps.
///
/// [`OrderedMap`]: crate::OrderedMap
/// [lexographical]: core::cmp::Ord#lexographical-comparison
pub struct UnorderedMap<K, V, S = ahash::RandomState> {
    pub(crate) inner: HashMap<K, V, S>,
}

impl<K, V> UnorderedMap<K, V, ahash::RandomState> {
    /// [`HashMap::new`] but using an [`ahash`] hasher.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// [`HashMap::with_capacity`] but using an [`ahash`] hasher.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, ahash::RandomState::default())
    }
}

impl<K, V, S> UnorderedMap<K, V, S> {
    /// See [`HashMap::with_capacity_and_hasher`]
    #[inline]
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        Self {
            inner: HashMap::<K, V, S>::with_capacity_and_hasher(capacity, hasher),
        }
    }

    /// See [`HashMap::len`]
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// See [`HashMap::capacity`]
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// See [`HashMap::keys`]
    #[inline]
    pub fn keys(&self) -> Keys<'_, K, V> {
        self.inner.keys()
    }

    /// See [`HashMap::into_keys`]
    #[inline]
    pub fn into_keys(self) -> IntoKeys<K, V> {
        self.inner.into_keys()
    }

    /// See [`HashMap::values`]
    #[inline]
    pub fn values(&self) -> Values<'_, K, V> {
        self.inner.values()
    }

    /// See [`HashMap::values_mut`]
    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        self.inner.values_mut()
    }

    /// See [`HashMap::into_values`]
    #[inline]
    pub fn into_values(self) -> IntoValues<K, V> {
        self.inner.into_values()
    }

    /// See [`HashMap::iter`]
    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        self.inner.iter()
    }

    /// See [`HashMap::iter_mut`]
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        self.inner.iter_mut()
    }

    /// See [`HashMap::is_empty`]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// See [`HashMap::drain`]
    #[inline]
    pub fn drain(&mut self) -> Drain<'_, K, V> {
        self.inner.drain()
    }

    /// See [`HashMap::retain`]
    #[inline]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        self.inner.retain(f)
    }

    /// See [`HashMap::clear`]
    ///
    /// Note that this method does not shrink the underlying allocation (keeps capacity the same) and is `O(capacity)`.
    /// Thus repeated calls to `clear` on a map that is far under-occupied may be unexpectedly expensive. Consider using
    /// [`clear_and_shrink`] or [`clear_and_shrink_to`] to shrink the underlying allocation when appropriate when clearing.
    ///
    /// [`clear_and_shrink`]: OrderedMap::clear_and_shrink
    /// [`clear_and_shrink_to`]: OrderedMap::clear_and_shrink_to
    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    /// See [`HashMap::hasher`]
    #[inline]
    pub fn hasher(&self) -> &S {
        self.inner.hasher()
    }

    /// Access the wrapped [`HashMap`].
    #[inline]
    pub fn inner(&self) -> &HashMap<K, V, S> {
        &self.inner
    }

    /// Access the wrapped [`HashMap`] mutably.
    #[inline]
    pub fn inner_mut(&mut self) -> &mut HashMap<K, V, S> {
        &mut self.inner
    }

    /// Extract the wrapped [`HashMap`].
    #[inline]
    pub fn into_inner(self) -> HashMap<K, V, S> {
        self.inner
    }
}

impl<K, V, const N: usize> From<[(K, V); N]> for UnorderedMap<K, V, ahash::RandomState>
where
    K: Hash + Eq,
{
    fn from(arr: [(K, V); N]) -> Self {
        Self {
            inner: HashMap::<K, V, ahash::RandomState>::from_iter(arr),
        }
    }
}

impl<K, V, S> UnorderedMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    /// See [`HashMap::reserve`]
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional)
    }

    /// See [`HashMap::try_reserve`]
    #[inline]
    pub fn try_reserve(
        &mut self,
        additional: usize,
    ) -> Result<(), std::collections::TryReserveError> {
        self.inner.try_reserve(additional)
    }

    /// See [`HashMap::shrink_to_fit`]
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit();
    }

    /// See [`HashMap::shrink_to`]
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.inner.shrink_to(min_capacity);
    }

    /// Clears and shrinks the capacity of the map on a basic heuristic. If you have a more specific heuristic, see [`clear_and_shrink_to`].
    ///
    /// If the map previously had > 128 element capacity, shrinks to whichever is larger between 128 and 110% of the previous length of the map
    /// in an effort to reduce reallocation for repeated use-and-clear on similar numbers of items. If the map had <= 128 element capacity, no shrink happens.
    ///
    /// [`clear_and_shrink_to`]: UnorderedMap::clear_and_shrink_to
    #[inline]
    pub fn clear_and_shrink(&mut self) {
        if self.capacity() > 128 {
            let new_cap = 128usize.max((self.len() as f64 * 1.1) as usize);
            self.clear_and_shrink_to(new_cap);
        } else {
            self.clear();
        }
    }

    /// Clears and shrinks the capacity of the map to the given capacity.
    #[inline]
    pub fn clear_and_shrink_to(&mut self, capacity: usize) {
        self.clear();
        self.shrink_to(capacity);
    }

    /// See [`HashMap::entry`]
    #[inline]
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        self.inner.entry(key)
    }

    /// See [`HashMap::get`]
    #[inline]
    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.inner.get(k)
    }

    /// See [`HashMap::get_key_value`]
    #[inline]
    pub fn get_key_value<Q: ?Sized>(&self, k: &Q) -> Option<(&K, &V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.inner.get_key_value(k)
    }

    /// See [`HashMap::get_mut`]
    #[inline]
    pub fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.inner.get_mut(k)
    }

    /// See [`HashMap::insert`]
    #[inline]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.inner.insert(k, v)
    }

    /// See [`HashMap::remove`]
    #[inline]
    pub fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.inner.remove(k)
    }

    /// See [`HashMap::contains_key`]
    #[inline]
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.inner.contains_key(key)
    }
}

impl<K, V, S> Clone for UnorderedMap<K, V, S>
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

impl<K, V, S> Default for UnorderedMap<K, V, S>
where
    S: BuildHasher + Default,
{
    #[inline]
    fn default() -> Self {
        Self {
            inner: HashMap::with_hasher(S::default()),
        }
    }
}

impl<K, V, S> fmt::Debug for UnorderedMap<K, V, S>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnorderedMap")
            .field("inner", &self.inner)
            .finish()
    }
}

impl<K, V, S> PartialEq for UnorderedMap<K, V, S>
where
    K: Eq + Hash,
    V: PartialEq,
    S: BuildHasher,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl<K, V, S> Eq for UnorderedMap<K, V, S>
where
    K: Eq + Hash,
    V: Eq,
    S: BuildHasher,
{
}

impl<K, V, S> PartialOrd for UnorderedMap<K, V, S>
where
    K: Eq + Hash + Ord,
    V: Ord,
    S: BuildHasher,
{
    #[inline]
    fn partial_cmp(&self, other: &UnorderedMap<K, V, S>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K, V, S> Ord for UnorderedMap<K, V, S>
where
    K: Eq + Hash + Ord,
    V: Ord,
    S: BuildHasher,
{
    fn cmp(&self, other: &UnorderedMap<K, V, S>) -> Ordering {
        // first compare lengths, if equal, we have to sort and do lexographical ordering...
        match self.len().cmp(&other.len()) {
            Ordering::Less => return Ordering::Less,
            Ordering::Greater => return Ordering::Greater,
            Ordering::Equal => (),
        }
        let mut self_seq = self.inner.iter().collect::<Vec<_>>();
        self_seq.sort_by_key(|(k, _v)| *k);
        let mut other_seq = other.inner.iter().collect::<Vec<_>>();
        other_seq.sort_by_key(|(k, _v)| *k);
        self_seq.into_iter().cmp(other_seq)
    }
}

impl<K, V, S> Hash for UnorderedMap<K, V, S>
where
    K: Hash,
    V: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        // although this is in theory not fully cryptographically secure, we don't care about that here.
        // also, since we are using a deterministic hasher, we don't have to worry about differences between maps' hashers.
        // Thus we can use xor as an order-independent hash-combination function. Thus, this
        // will create the same hash as long as two graphs have the same element (k, v) pairs, regardless of order.
        // this also satisfies the requirement that Eq and Hash have the same semantics, as HashMap's PartialEq and Eq
        // also have this semantics that as long as the elements are equal the maps are equal.
        let mut hash = 0u64;
        for elt in self.inner.iter() {
            let elt_hash = STATIC_RANDOM_STATE.hash_one(elt);
            hash ^= elt_hash;
        }
        state.write_u64(hash);
    }
}

impl<K, V, S> FromIterator<(K, V)> for UnorderedMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher + Default,
{
    #[inline]
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self {
            inner: HashMap::<K, V, S>::from_iter(iter),
        }
    }
}

impl<K, V, S> IntoIterator for UnorderedMap<K, V, S> {
    type Item = (K, V);
    type IntoIter = std::collections::hash_map::IntoIter<K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a UnorderedMap<K, V, S> {
    type Item = (&'a K, &'a V);
    type IntoIter = std::collections::hash_map::Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut UnorderedMap<K, V, S> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = std::collections::hash_map::IterMut<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}

#[cfg(feature = "serde")]
impl<K, V, RS> serde::Serialize for UnorderedMap<K, V, RS>
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
impl<'de, K, V, S> serde::Deserialize<'de> for UnorderedMap<K, V, S>
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

        struct HashMapSeqVisitor<K, V, S>(PhantomData<(K, V, S)>);

        impl<'de, K, V, S> Visitor<'de> for HashMapSeqVisitor<K, V, S>
        where
            K: serde::Deserialize<'de> + Eq + Hash,
            V: serde::Deserialize<'de>,
            S: BuildHasher + Default,
        {
            type Value = HashMap<K, V, S>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a sequence of (k, v) pairs")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut map =
                    HashMap::with_capacity_and_hasher(seq.size_hint().unwrap_or(0), S::default());

                while let Some((k, v)) = seq.next_element::<(K, V)>()? {
                    map.insert(k, v);
                }

                Ok(map)
            }
        }

        let map = deserializer.deserialize_seq(HashMapSeqVisitor::<K, V, S>(PhantomData))?;
        Ok(Self { inner: map })
    }
}

#[cfg(feature = "speedy")]
impl<'a, C, K, V, S> speedy::Readable<'a, C> for UnorderedMap<K, V, S>
where
    C: speedy::Context,
    K: speedy::Readable<'a, C> + Eq + Hash,
    V: speedy::Readable<'a, C>,
    S: BuildHasher + Default,
{
    fn read_from<R: speedy::Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let map = HashMap::<K, V, S>::read_from(reader)?;
        Ok(Self { inner: map })
    }
}

#[cfg(feature = "speedy")]
impl<C, K, V, S> speedy::Writable<C> for UnorderedMap<K, V, S>
where
    C: speedy::Context,
    K: speedy::Writable<C>,
    V: speedy::Writable<C>,
{
    fn write_to<T: ?Sized + speedy::Writer<C>>(
        &self,
        writer: &mut T,
    ) -> Result<(), <C as speedy::Context>::Error> {
        HashMap::<K, V, S>::write_to(&self.inner, writer)
    }
}
