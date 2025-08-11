extern crate alloc;
use alloc::{
    borrow::{Cow, ToOwned},
    boxed::Box,
    collections::{BTreeMap, BTreeSet, LinkedList, VecDeque},
    // gated to rustc 1.64: ffi::CString,
    rc::Rc,
    string::String,
    sync::Arc,
    vec::Vec,
};

#[rustversion::since(1.64)]
use alloc::ffi::CString;

use crate::{PortableHash, PortableHasher, PortableOrd};

impl<T: ToOwned + PortableHash> PortableHash for Cow<'_, T> {
    fn portable_hash<H: PortableHasher>(&self, state: &mut H) {
        (&self).portable_hash(state);
    }
}

impl<T: PortableHash> PortableHash for Box<T> {
    #[inline]
    fn portable_hash<H: PortableHasher>(&self, state: &mut H) {
        (**self).portable_hash(state);
    }
}

impl<T: PortableHash> PortableHash for Rc<T> {
    #[inline]
    fn portable_hash<H: PortableHasher>(&self, state: &mut H) {
        (**self).portable_hash(state);
    }
}

impl<T: PortableHash> PortableHash for Arc<T> {
    #[inline]
    fn portable_hash<H: PortableHasher>(&self, state: &mut H) {
        (**self).portable_hash(state);
    }
}

impl<K, V> PortableHash for BTreeMap<K, V>
where
    K: PortableHash + PortableOrd,
    V: PortableHash,
{
    #[inline]
    fn portable_hash<H: PortableHasher>(&self, state: &mut H) {
        state.write_len_prefix(self.len());
        for (key, value) in self {
            key.portable_hash(state);
            value.portable_hash(state);
        }
    }
}

impl<K> PortableHash for BTreeSet<K>
where
    K: PortableHash + PortableOrd,
{
    #[inline]
    fn portable_hash<H: PortableHasher>(&self, state: &mut H) {
        state.write_len_prefix(self.len());
        for item in self {
            item.portable_hash(state);
        }
    }
}

impl<T> PortableHash for LinkedList<T>
where
    T: PortableHash,
{
    #[inline]
    fn portable_hash<H: PortableHasher>(&self, state: &mut H) {
        state.write_len_prefix(self.len());
        for item in self {
            item.portable_hash(state);
        }
    }
}

impl<T> PortableHash for VecDeque<T>
where
    T: PortableHash,
{
    #[inline]
    fn portable_hash<H: PortableHasher>(&self, state: &mut H) {
        state.write_len_prefix(self.len());
        for item in self {
            item.portable_hash(state);
        }
    }
}

#[rustversion::since(1.64)]
impl PortableHash for CString {
    #[inline]
    fn portable_hash<H: PortableHasher>(&self, state: &mut H) {
        (&**self).portable_hash(state);
    }
}

impl PortableHash for String {
    #[inline]
    fn portable_hash<H: PortableHasher>(&self, state: &mut H) {
        self.as_str().portable_hash(state);
    }
}

impl<T: PortableHash> PortableHash for Vec<T> {
    #[inline]
    fn portable_hash<H: PortableHasher>(&self, state: &mut H) {
        self.as_slice().portable_hash(state);
    }
}
