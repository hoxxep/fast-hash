#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]
#![cfg_attr(not(feature = "std"), no_std)]

// #![warn(missing_docs)]
#![deny(unused_must_use)]

mod hash_impls;

pub use fast_hash_macros::FastHash;

/// A faster Hash trait with a few optimizations:
/// - `write_bytes`, `write_str`, and `write_len_prefix` for optimising the length prefix in
///   non-bytewise hash functions.
/// - Move the `write_len_prefix` method out of the array impl, and into hash_slice.
/// - `HASH_AS_BYTES` constant to indicate if the type can be hashed as a single write call.
///
/// TODO: A big drawback is the need for unsafe code to transmute arbitrary types to byte slices, which
/// may trip some `#![deny(unsafe)]`. The user also has to implement the `if Self::HASH_AS_BYTES`
/// logic themselves, which they may also forget to do. We could offer a wrapper to `hash` and
/// also other methods to safely transmute to byte slices where type: Sized, but it's not a
/// perfect fix.
pub trait FastHash {
    /// We can simply bulk hash this type as a singe `write` call if:
    /// - This type and subtypes are all unpadded.
    /// - This type and subtypes are not comprised of any pointers or references.
    ///
    /// We can generally check for padding by asserting:
    /// - size_of::<Self>() == sum(size_of::<subtypes>())
    /// - and, all subtypes are HASH_AS_BYTES = true.
    ///
    /// The layout may change between platforms and compiler versions, so this is never portable,
    /// but it can be much, much faster for hashing collections of unpadded types.
    const HASH_AS_BYTES: bool = false;

    /// Computes the hash of the object using the specified hasher.
    ///
    /// TODO: the hash bytes logic ends up being copied into every hash impl, and requires a user
    ///   to write some unsafe code. Should there be a wrapper method that does this?
    fn hash<H: FastHasher>(&self, state: &mut H);

    /// Feed a slice of this type into the given [`Hasher`].
    #[inline]
    fn hash_slice<H: FastHasher>(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        if Self::HASH_AS_BYTES {
            // if we can hash this type as a single write, we do so instead
            let newlen = size_of_val(data);
            let ptr = data.as_ptr() as *const u8;

            // SAFETY: This is safe because we assume that the type is unpadded and does not contain
            // any pointers or references, so it can be safely treated as a byte slice.
            state.write_bytes(unsafe { core::slice::from_raw_parts(ptr, newlen) })
        } else {
            // otherwise, we hash each item individually
            state.write_len_prefix(data.len());
            for item in data.iter() {
                item.hash(state);
            }
        }
    }

    #[inline]
    fn hash_as_bytes<H: FastHasher>(data: &Self, state: &mut H)
    where
        Self: Sized,
    {
        debug_assert!(Self::HASH_AS_BYTES);

        // if we can hash this type as a single write, we do so instead
        let newlen = core::mem::size_of_val(data);
        let ptr = data as *const Self as *const u8;

        // SAFETY: This is safe because we assume that the type is unpadded and does not contain
        // any pointers or references, so it can be safely treated as a byte slice.
        state.write_bytes(unsafe { core::slice::from_raw_parts(ptr, newlen) });
    }
}

// pub trait FashHashAsBytes: Sized {
//     /// Hash this type as a single bytes write.
//     ///
//     /// We call write() directly, as the type is a fixed size, and so a length prefix is not
//     /// required.
//     fn hash_as_bytes<H: FastHasher>(&self, state: &mut H) {
//         let len = size_of::<H>();
//         let ptr = self as *const Self as *const u8;
//
//         // SAFETY: This is safe because we're viewing an allocated type as a byte slice.
//         state.write(unsafe { core::slice::from_raw_parts(ptr, len) });
//     }
//
//     /// If we can hash this type as a single write, we do so. The size of a type is [always a
//     /// multiple of its alignment](https://doc.rust-lang.org/reference/type-layout.html#size-and-alignment),
//     /// and so a slice should not introduce any padding.
//     ///
//     /// We call `write_bytes()` to allow the hasher to add a length prefix if required.
//     fn hash_slice_as_bytes<H: FastHasher>(data: &[Self], state: &mut H) {
//         let newlen = size_of::<H>();
//         let ptr = data.as_ptr() as *const u8;
//
//         // SAFETY: This is safe because we're viewing an allocated type as a byte slice.
//         state.write_bytes(unsafe { core::slice::from_raw_parts(ptr, newlen) })
//     }
// }
//
// impl<T: FashHashAsBytes> FastHash for T {
//     const HASH_AS_BYTES: bool = true;
//
//     #[inline]
//     fn hash<H: FastHasher>(&self, state: &mut H) {
//         Self::hash_as_bytes(self, state);
//     }
//
//     #[inline]
//     fn hash_slice<H: FastHasher>(data: &[Self], state: &mut H) {
//         Self::hash_slice_as_bytes(data, state);
//     }
// }

pub trait FastHasher {
    fn finish(&self) -> u64;
    fn write(&mut self, bytes: &[u8]);

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.write(&[i]);
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.write(&i.to_ne_bytes());
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.write(&i.to_ne_bytes());
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.write(&i.to_ne_bytes());
    }

    #[inline]
    fn write_u128(&mut self, i: u128) {
        self.write(&i.to_ne_bytes());
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.write(&i.to_ne_bytes());
    }

    #[inline]
    fn write_i8(&mut self, i: i8) {
        self.write_u8(i as u8);
    }

    #[inline]
    fn write_i16(&mut self, i: i16) {
        self.write_u16(i as u16);
    }

    #[inline]
    fn write_i32(&mut self, i: i32) {
        self.write_u32(i as u32);
    }

    #[inline]
    fn write_i64(&mut self, i: i64) {
        self.write_u64(i as u64);
    }

    #[inline]
    fn write_i128(&mut self, i: i128) {
        self.write_u128(i as u128);
    }

    #[inline]
    fn write_isize(&mut self, i: isize) {
        self.write_usize(i as usize);
    }

    /// Write a length prefix before writing the bytes.
    #[inline]
    fn write_len_prefix(&mut self, len: usize) {
        // Write the length as a u64.
        self.write_usize(len);
    }

    /// Write a string.
    #[inline]
    fn write_str(&mut self, s: &str) {
        // Write the length prefix.
        self.write_len_prefix(s.len());
        // Write the bytes of the string.
        self.write(s.as_bytes());
    }

    /// Write a slice of bytes.
    #[inline]
    fn write_bytes(&mut self, bytes: &[u8]) {
        // Write the length prefix.
        self.write_len_prefix(bytes.len());
        // Write the bytes.
        self.write(bytes);
    }
}

trait IsTrue<const COND: bool> {}
impl IsTrue<true> for () {}
