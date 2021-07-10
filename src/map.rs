//! Defines a trait for key-value map types.

use std::borrow::Borrow;
use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;

/// A type that associates a value with a key.
pub trait Map<Q, V>
where
    Q: Hash + Eq + ?Sized,
{
    /// Returns a reference to the value associated with the given key, if any.
    fn get(&self, key: &Q) -> Option<&V>;
}

impl<K, Q, V> Map<Q, V> for HashMap<K, V>
where
    Q: Hash + Eq + ?Sized,
    K: Borrow<Q> + Hash + Eq,
{
    fn get(&self, key: &Q) -> Option<&V> {
        <HashMap<K, V>>::get(self, key)
    }
}

impl<K, Q, V> Map<Q, V> for HashMap<K, &V>
where
    Q: Hash + Eq + ?Sized,
    K: Borrow<Q> + Hash + Eq,
{
    fn get(&self, key: &Q) -> Option<&V> {
        <HashMap<K, &V>>::get(self, key).map(|v| *v)
    }
}

/// A `Map` implementation that always returns `None`.
pub struct NoMap;

impl<Q, V> Map<Q, V> for NoMap
where
    Q: Hash + Eq + ?Sized,
{
    fn get(&self, _: &Q) -> Option<&V> {
        None
    }
}
