use core::any::{Any, TypeId};
use core::fmt;

use alloc::boxed::Box;

type FxHashMap<K, V> =
    hashbrown::HashMap<K, V, core::hash::BuildHasherDefault<rustc_hash::FxHasher>>;

#[derive(Default)]
/// A type map of extensions.
pub struct Extensions {
    map: FxHashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Extensions {
    /// Create an empty `Extensions`.
    #[inline]
    pub fn new() -> Extensions {
        Extensions {
            map: Default::default(),
        }
    }
    /// Insert a value into this `Extensions`.
    ///
    /// If a extension of this type already exists, it will
    /// be returned.
    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        self.map
            .insert(TypeId::of::<T>(), Box::new(val))
            .and_then(|boxed| {
                (boxed as Box<dyn Any + 'static>)
                    .downcast()
                    .ok()
                    .map(|boxed| *boxed)
            })
    }

    /// Check if container contains typed value
    pub fn contains<T: Send + Sync + 'static>(&self) -> bool {
        self.map.get(&TypeId::of::<T>()).is_some()
    }

    /// Get a reference to a typed value previously inserted on this `Extensions`.
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|boxed| (&**boxed as &(dyn Any + 'static)).downcast_ref())
    }

    /// Get a mutable reference to a typed value previously inserted on this `Extensions`.
    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.map
            .get_mut(&TypeId::of::<T>())
            .and_then(|boxed| (&mut **boxed as &mut (dyn Any + 'static)).downcast_mut())
    }

    /// Remove a typed value from this `Extensions`.
    ///
    /// If a value of this type exists, it will be returned.
    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.map.remove(&TypeId::of::<T>()).and_then(|boxed| {
            (boxed as Box<dyn Any + 'static>)
                .downcast()
                .ok()
                .map(|boxed| *boxed)
        })
    }

    /// Clear the `Extensions` of all inserted values.
    #[inline]
    pub fn clear(&mut self) {
        self.map.clear();
    }
}

impl fmt::Debug for Extensions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Extensions").finish()
    }
}
