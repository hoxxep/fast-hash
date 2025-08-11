//! [`PortableHash`] implementations for primitive types and core types.
//!
//! Modified from the `std::hash` module in the rust standard library.

use crate::{FastHash, FastHasher};

macro_rules! impl_hash_int {
    ($(($ty:ident, $meth:ident),)*) => {$(

        /// We differ from the standard library implementation here, as we do _not_ allow
        /// [`portable_hash_slice`] to transmute unpadded numbers into a byte slice. This is
        /// because we need to control the endianness of the numbers when hashing, and a simple
        /// byte slice would not be portable across platforms. Likewise, `usize` and `isize` are
        /// not portable across platforms as a byte slice.
        impl FastHash for $ty {
            const HASH_AS_BYTES: bool = true;

            #[inline]
            fn hash<H: FastHasher>(&self, state: &mut H) {
                state.$meth(*self)
            }
        }
    )*}
}

impl_hash_int! {
    (u8, write_u8),
    (u16, write_u16),
    (u32, write_u32),
    (u64, write_u64),
    (u128, write_u128),
    (usize, write_usize),
    (i8, write_i8),
    (i16, write_i16),
    (i32, write_i32),
    (i64, write_i64),
    (i128, write_i128),
    (isize, write_isize),
}

impl FastHash for bool {
    // TODO: check bool is guaranteed to be 0/1
    const HASH_AS_BYTES: bool = true;

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        state.write_u8(*self as u8)
    }
}

impl FastHash for char {
    const HASH_AS_BYTES: bool = true;

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        state.write_u32(*self as u32)
    }
}

impl FastHash for str {
    /// This applies to `[str]`, not `[&str]`.
    const HASH_AS_BYTES: bool = true;

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        state.write_str(self);
    }
}

macro_rules! impl_hash_tuple {
    () => (

        impl FastHash for () {
            // there is nothing to hash, instead write_len_prefix
            const HASH_AS_BYTES: bool = core::mem::size_of::<()>() == 0;

            #[inline]
            fn hash<H: FastHasher>(&self, _state: &mut H) {}
        }
    );

    ($($name:ident)+) => (
        maybe_tuple_doc! {
            $($name)+ @

            impl<$($name: FastHash),+> FastHash for ($($name,)+) {  // TODO: support where last_type!($($name,)+): ?Sized
                const HASH_AS_BYTES: bool = {
                    let self_size = core::mem::size_of::<($($name,)+)>();
                    let item_size = $(core::mem::size_of::<$name>() + )+ 0;
                    let all_as_bytes = $($name::HASH_AS_BYTES && )+ true;
                    self_size == item_size && all_as_bytes
                };

                #[allow(non_snake_case)]
                #[inline]
                fn hash<S: FastHasher>(&self, state: &mut S) {
                    if Self::HASH_AS_BYTES {
                        // if we can hash this tuple as a single write, we do so instead
                        let newlen = core::mem::size_of_val(self);
                        let ptr = self as *const _ as *const u8;

                        // we don't use write_bytes, as we're writing a "single type" of fixed size,
                        // so there is no write_len_prefix call necessary.

                        // SAFETY: This is safe because we assume that the tuple is unpadded and does not contain
                        // any pointers or references, so it can be safely treated as a byte slice.
                        state.write(unsafe { core::slice::from_raw_parts(ptr, newlen) });
                    } else {
                        // otherwise, we hash each item individually
                        let ($(ref $name,)+) = *self;
                        $($name.hash(state);)+
                    }
                }
            }
        }
    );
}

macro_rules! maybe_tuple_doc {
    ($a:ident @ $item:item) => {
        #[doc = "This trait is implemented for tuples up to twelve items long."]
        $item
    };
    ($a:ident $($rest_a:ident)+ @ $item:item) => {
        #[doc(hidden)]
        $item
    };
}

macro_rules! last_type {
    ($a:ident,) => { $a };
    ($a:ident, $($rest_a:ident,)+) => { last_type!($($rest_a,)+) };
}

impl_hash_tuple! {}
impl_hash_tuple! { T }
impl_hash_tuple! { T B }
impl_hash_tuple! { T B C }
impl_hash_tuple! { T B C D }
impl_hash_tuple! { T B C D E }
impl_hash_tuple! { T B C D E F }
impl_hash_tuple! { T B C D E F G }
impl_hash_tuple! { T B C D E F G H }
impl_hash_tuple! { T B C D E F G H I }
impl_hash_tuple! { T B C D E F G H I J }
impl_hash_tuple! { T B C D E F G H I J K }
impl_hash_tuple! { T B C D E F G H I J K L }

impl<T: FastHash> FastHash for [T] {
    const HASH_AS_BYTES: bool = {
        let self_size = core::mem::size_of::<[T; 2]>();
        let item_size = core::mem::size_of::<T>() * 2;
        let all_as_bytes = T::HASH_AS_BYTES;
        self_size == item_size && all_as_bytes
    };

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        FastHash::hash_slice(self, state)
    }
}

impl<T: FastHash, const LEN: usize> FastHash for [T; LEN] {
    const HASH_AS_BYTES: bool = {
        let self_size = core::mem::size_of::<[T; LEN]>();
        let item_size = core::mem::size_of::<T>() * LEN;
        let all_as_bytes = T::HASH_AS_BYTES;
        self_size == item_size && all_as_bytes
    };

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        FastHash::hash_slice(self.as_slice(), state)
    }
}

impl<T: ?Sized + FastHash> FastHash for &T {
    // must hash the referenced value, not the reference itself
    const HASH_AS_BYTES: bool = false;

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T: ?Sized + FastHash> FastHash for &mut T {
    // must hash the referenced value, not the reference itself
    const HASH_AS_BYTES: bool = false;

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_bytes() {
        // basic integers and references
        assert!(u64::HASH_AS_BYTES);
        assert!(!<&u64 as FastHash>::HASH_AS_BYTES);
        assert!(!<&mut u64 as FastHash>::HASH_AS_BYTES);

        // other basic types
        assert!(bool::HASH_AS_BYTES);
        assert!(char::HASH_AS_BYTES);
        assert!(str::HASH_AS_BYTES);

        // arrays and slices, the reference must get resolved separately!
        assert!(<[u64; 4] as FastHash>::HASH_AS_BYTES);
        assert!(<[u64] as FastHash>::HASH_AS_BYTES);
        assert!(!<&[u64] as FastHash>::HASH_AS_BYTES);
        assert!(!<&mut [u64] as FastHash>::HASH_AS_BYTES);

        // tuples with and without padding
        assert!(<() as FastHash>::HASH_AS_BYTES);
        assert!(<(u64, u64) as FastHash>::HASH_AS_BYTES);
        assert!(<(u8, u8, u16) as FastHash>::HASH_AS_BYTES);
        assert!(!<(u64, u32) as FastHash>::HASH_AS_BYTES);

        // tuples of arrays
        assert!(<(u64, [u64; 4]) as FastHash>::HASH_AS_BYTES);
        assert!(<(u64, [u32; 4]) as FastHash>::HASH_AS_BYTES);
        assert!(!<(u64, [u32; 3]) as FastHash>::HASH_AS_BYTES);
        assert!(<([u8; 8], [u64; 2]) as FastHash>::HASH_AS_BYTES);
    }
}
