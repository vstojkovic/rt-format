use std::borrow::Borrow;
use std::cmp::Eq;
use std::collections::HashMap;
use std::hash::Hash;

pub trait Map<Q, V>
where
    Q: Hash + Eq + ?Sized,
{
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

pub struct NoMap;

impl<Q, V> Map<Q, V> for NoMap
where
    Q: Hash + Eq + ?Sized,
{
    fn get(&self, _: &Q) -> Option<&V> {
        None
    }
}
