use std::collections::hash_map::Entry;
use std::{collections::HashMap, hash::Hash};

// Keys can only be created inside this module to prevent misuse.
pub trait KeyType {
    fn new(index: usize) -> Self;
    fn max_values() -> usize;
    fn as_index(&self) -> usize;
}

/// Creates a newtype named name to be used with an instance of KeyMap. These keys are
/// 16-bits.
macro_rules! key16 {
    ($name:ident) => {
        #[derive(Copy, Clone, Debug, Eq, PartialEq)]
        pub struct $name(u16);

        impl crate::utils::KeyType for $name {
            fn new(index: usize) -> Self {
                Self(index as u16)
            }

            fn max_values() -> usize {
                u16::MAX as usize
            }

            fn as_index(&self) -> usize {
                self.0 as usize
            }
        }
    };
}
pub(crate) use key16;

/// Hash map where the keys are small integers that are handed out when values are
/// inserted. Keys do not become invalidated because deletion is not supported. This is
/// similar to the SlotMap crate but optimized for use cases where a large number of keys
/// are in use.
#[derive(Clone)]
pub struct KeyMap<K, V>
where
    K: Copy + KeyType,
    V: Clone + Eq + Hash,
{
    by_key: Vec<V>,
    to_key: HashMap<V, K>,
}

impl<K: Copy + KeyType, V: Clone + Eq + Hash> KeyMap<K, V> {
    /// When too many values are inserted for the key type overflow is returned when get()
    /// is used for those values.
    pub fn new(overflow: V) -> Self {
        let mut result = Self {
            by_key: Vec::new(),
            to_key: HashMap::new(),
        };
        result.insert(overflow);
        result
    }

    /// Returns an existing key or creates a new one.
    pub fn insert(&mut self, value: V) -> K {
        match self.to_key.entry(value) {
            Entry::Occupied(occupied) => *occupied.get(),
            Entry::Vacant(vacant) => {
                if self.by_key.len() < K::max_values() {
                    let key = K::new(self.by_key.len());
                    self.by_key.push(vacant.key().clone());
                    vacant.insert(key);
                    key
                } else {
                    K::new(0)
                }
            }
        }
    }

    /// Returns the value for a key. This will never fail because keys are managed
    /// internally and values are never removed.
    pub fn get(&self, key: K) -> &V {
        &self.by_key[key.as_index()]
    }

    pub fn iter(&self) -> impl Iterator<Item = &V> {
        self.by_key.iter().skip(1)
    }
}
