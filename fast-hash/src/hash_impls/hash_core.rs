use core::{
    // explicitly omitted: alloc::Layout,
    // explicitly omitted: any::TypeId,
    cmp::{Ordering, Reverse},
    convert::Infallible,
    // gated to rustc 1.64: ffi::CStr,
    // explicitly omitted: fmt::Error,
    marker::{PhantomData, PhantomPinned},
    mem::{Discriminant, ManuallyDrop},
    num::{
        NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
        NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize, /* gated to 1.74 Saturating, */
        Wrapping,
    },
    ops::{
        Bound, ControlFlow, Deref, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo,
        RangeToInclusive,
    },
    // explicitly omitted: panic::Location,
    pin::Pin,
    // TODO: ptr::NonNull, (can we safely hash this?)
    // TODO: sync::atomic, (issues with Ordering stability, gating by available atomics, and what ordering to choose)
    task::Poll,
    time::Duration,
};

#[rustversion::since(1.64)]
use core::ffi::CStr;

#[rustversion::since(1.74)]
use core::num::Saturating;

use core::mem::size_of;
use crate::{FastHash, FastHasher};

impl<T: FastHash> FastHash for Option<T> {
    // None only writes the discriminant, the rest of the bytes may be arbitrary
    // TODO: support NonZero ints to HASH_AS_BYTES somehow?
    const HASH_AS_BYTES: bool = false;

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        FastHash::hash(&core::mem::discriminant(self), state);
        match self {
            Some(value) => value.hash(state),
            None => (),
        }
    }
}

impl<T: FastHash, E: FastHash> FastHash for Result<T, E> {
    // TODO: T and E types with the same size?
    const HASH_AS_BYTES: bool = false;

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        FastHash::hash(&core::mem::discriminant(self), state);
        match self {
            Ok(value) => value.hash(state),
            Err(err) => err.hash(state),
        }
    }
}

impl FastHash for Ordering {
    // If this enum is only one byte, then it's simply storing the discriminant.
    const HASH_AS_BYTES: bool = size_of::<Ordering>() == size_of::<Discriminant<Ordering>>() && Discriminant::<Ordering>::HASH_AS_BYTES;

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        FastHash::hash(&core::mem::discriminant(self), state);
    }
}

impl<T: FastHash> FastHash for Reverse<T> {
    const HASH_AS_BYTES: bool = T::HASH_AS_BYTES && size_of::<Self>() == size_of::<T>();

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl FastHash for Infallible {
    const HASH_AS_BYTES: bool = size_of::<Infallible>() == 0;

    fn hash<H: FastHasher>(&self, _state: &mut H) {
        // do nothing, as Infallible cannot be instantiated.
    }
}

#[rustversion::since(1.64)]
impl FastHash for CStr {
    /// This type is a pointer like `&str`, not the actual bytes.
    const HASH_AS_BYTES: bool = false;

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        // TODO: use write_bytes without nuLl? write_str?
        state.write(self.to_bytes_with_nul());
    }
}

impl<T: FastHash> FastHash for PhantomData<T> {
    const HASH_AS_BYTES: bool = size_of::<PhantomData<T>>() == 0;

    fn hash<H: FastHasher>(&self, _state: &mut H) {
        // do nothing, as PhantomData does not hold any data.
    }
}

impl FastHash for PhantomPinned {
    const HASH_AS_BYTES: bool = size_of::<PhantomPinned>() == 0;

    fn hash<H: FastHasher>(&self, _state: &mut H) {
        // do nothing, as PhantomPinned does not hold any data.
    }
}

// TODO: The Discriminant may not need T to be FastHash?
impl<T> FastHash for Discriminant<T> {
    /// TODO: Discriminant<T>
    const HASH_AS_BYTES: bool = true;

    #[inline]
    fn hash<H: FastHasher>(&self, _state: &mut H) {
        todo!("Discriminant<T> is not yet implemented for FastHash, needs forwarding from core");
    }
}

impl<T: FastHash> FastHash for ManuallyDrop<T> {
    const HASH_AS_BYTES: bool = T::HASH_AS_BYTES;

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}

macro_rules! impl_non_zero {
    ($int:ident, $method:ident) => {
        impl FastHash for $int {
            const HASH_AS_BYTES: bool = true;

            #[inline]
            fn hash<H: FastHasher>(&self, state: &mut H) {
                state.$method(self.get());
            }
        }
    };
}

impl_non_zero!(NonZeroI8, write_i8);
impl_non_zero!(NonZeroI16, write_i16);
impl_non_zero!(NonZeroI32, write_i32);
impl_non_zero!(NonZeroI64, write_i64);
impl_non_zero!(NonZeroI128, write_i128);
impl_non_zero!(NonZeroIsize, write_isize);
impl_non_zero!(NonZeroU8, write_u8);
impl_non_zero!(NonZeroU16, write_u16);
impl_non_zero!(NonZeroU32, write_u32);
impl_non_zero!(NonZeroU64, write_u64);
impl_non_zero!(NonZeroU128, write_u128);
impl_non_zero!(NonZeroUsize, write_usize);

#[rustversion::since(1.74)]
impl<T: FastHash> FastHash for Saturating<T> {
    const HASH_AS_BYTES: bool = T::HASH_AS_BYTES && size_of::<Self>() == size_of::<T>();

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T: FastHash> FastHash for Wrapping<T> {
    const HASH_AS_BYTES: bool = T::HASH_AS_BYTES && size_of::<Self>() == size_of::<T>();

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T: FastHash> FastHash for Bound<T> {
    const HASH_AS_BYTES: bool = false;

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        // TODO: match std
        FastHash::hash(&core::mem::discriminant(self), state);
        match self {
            Bound::Included(value) => value.hash(state),
            Bound::Excluded(value) => value.hash(state),
            Bound::Unbounded => {}
        }
    }
}

impl<B: FastHash, C: FastHash> FastHash for ControlFlow<B, C> {
    /// If both B and C are the same size, and the discriminant is only a single byte, and there's
    /// no extra padding... Then this works?
    // TODO: review this
    const HASH_AS_BYTES: bool =
        B::HASH_AS_BYTES && C::HASH_AS_BYTES && Discriminant::<Self>::HASH_AS_BYTES &&
        (size_of::<B>() == size_of::<C>()) &&
        (size_of::<Self>() <= (size_of::<Discriminant::<Self>>() + size_of::<B>()));

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        FastHash::hash(&core::mem::discriminant(self), state);
        match self {
            ControlFlow::Continue(value) => value.hash(state),
            ControlFlow::Break(value) => value.hash(state),
        }
    }
}

impl<T: FastHash> FastHash for Range<T> {
    const HASH_AS_BYTES: bool = T::HASH_AS_BYTES && size_of::<Self>() == 2 * size_of::<T>();

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        self.start.hash(state);
        self.end.hash(state);
    }
}

impl<T: FastHash> FastHash for RangeFrom<T> {
    const HASH_AS_BYTES: bool = T::HASH_AS_BYTES && size_of::<Self>() == size_of::<T>();

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        self.start.hash(state);
    }
}

impl FastHash for RangeFull {
    const HASH_AS_BYTES: bool = true;

    #[inline]
    fn hash<H: FastHasher>(&self, _state: &mut H) {
        // RangeFull has no data to hash
    }
}

impl<T: FastHash> FastHash for RangeInclusive<T> {
    // the standard library reserves the right to change the implementation of RangeInclusive.
    const HASH_AS_BYTES: bool = false;

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        self.start().hash(state);
        self.end().hash(state);
    }
}

impl<T: FastHash> FastHash for RangeTo<T> {
    const HASH_AS_BYTES: bool = T::HASH_AS_BYTES && size_of::<Self>() == size_of::<T>();

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        self.end.hash(state);
    }
}

impl<T: FastHash> FastHash for RangeToInclusive<T> {
    const HASH_AS_BYTES: bool = T::HASH_AS_BYTES && size_of::<T>() == size_of::<T>();

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        self.end.hash(state);
    }
}

impl<T: Deref<Target = impl FastHash> + FastHash> FastHash for Pin<T> {
    /// Pin is internally a pointer-like type, so we hash the dereferenced value.
    const HASH_AS_BYTES: bool = false;

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        FastHash::hash(self.deref(), state);
    }
}

// macro_rules! impl_atomic {
//     ($int:ident, $method:ident) => {
//         impl PortableHash for atomic::$int {
//             #[inline]
//             fn portable_hash<H: PortableHasher>(&self, state: &mut H) {
//                 // TODO(stabilisation): Remove this implementation as ordering is application-dependent?
//                 state.$method(self.load(atomic::Ordering::SeqCst));
//             }
//         }
//     };
// }
//
// TODO: feature-gate based on availability of atomic types.
// impl_atomic!(AtomicI8, write_i8);
// impl_atomic!(AtomicI16, write_i16);
// impl_atomic!(AtomicI32, write_i32);
// impl_atomic!(AtomicI64, write_i64);
// impl_atomic!(AtomicIsize, write_isize);
// impl_atomic!(AtomicU8, write_u8);
// impl_atomic!(AtomicU16, write_u16);
// impl_atomic!(AtomicU32, write_u32);
// impl_atomic!(AtomicU64, write_u64);
// impl_atomic!(AtomicUsize, write_usize);

// impl PortableHash for atomic::Ordering {
//     // TODO(stabilisation): Consider removing this method if atomic orderings aren't stable.
//     #[inline]
//     fn portable_hash<H: PortableHasher>(&self, state: &mut H) {
//         match self {
//             atomic::Ordering::Relaxed => state.write_u8(0),
//             atomic::Ordering::SeqCst => state.write_u8(1),
//             atomic::Ordering::Acquire => state.write_u8(2),
//             atomic::Ordering::Release => state.write_u8(3),
//             atomic::Ordering::AcqRel => state.write_u8(4),
//             _ => panic!("Currently unsupported atomic ordering. Please raise a github issue."),
//         }
//     }
// }

impl<T: FastHash> FastHash for Poll<T> {
    const HASH_AS_BYTES: bool = false;

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        FastHash::hash(&core::mem::discriminant(self), state);
        match self {
            Poll::Pending => (),
            Poll::Ready(inner) => inner.hash(state),
        }
    }
}

impl FastHash for Duration {
    const HASH_AS_BYTES: bool = false;

    #[inline]
    fn hash<H: FastHasher>(&self, state: &mut H) {
        state.write_u64(self.as_secs());
        state.write_u32(self.subsec_nanos());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_bytes() {
        assert!(!Option::<u64>::HASH_AS_BYTES);
        assert!(!Result::<u64, u64>::HASH_AS_BYTES);
        assert!(Ordering::HASH_AS_BYTES);
        assert!(Reverse::<u64>::HASH_AS_BYTES);
        assert!(!Reverse::<&u64>::HASH_AS_BYTES);
        assert!(!Reverse::<(u64, u32)>::HASH_AS_BYTES);
        assert!(Infallible::HASH_AS_BYTES);
        assert!(PhantomData::<u64>::HASH_AS_BYTES);
        assert!(PhantomData::<&u64>::HASH_AS_BYTES);
        assert!(PhantomPinned::HASH_AS_BYTES);
        assert!(Discriminant::<Option<u64>>::HASH_AS_BYTES);
        assert!(ManuallyDrop::<u64>::HASH_AS_BYTES);
        assert!(NonZeroI8::HASH_AS_BYTES);
        assert!(NonZeroU64::HASH_AS_BYTES);
        assert!(Wrapping::<u64>::HASH_AS_BYTES);
        assert!(!Bound::<u64>::HASH_AS_BYTES);
        assert!(ControlFlow::<u64, u64>::HASH_AS_BYTES);
        assert!(!ControlFlow::<u64, u32>::HASH_AS_BYTES);
        assert!(!ControlFlow::<&u64, &u64>::HASH_AS_BYTES);
        assert!(Range::<u64>::HASH_AS_BYTES);
        assert!(!Range::<&u64>::HASH_AS_BYTES);
        assert!(RangeFrom::<u64>::HASH_AS_BYTES);
        assert!(RangeFull::HASH_AS_BYTES);
        assert!(!RangeInclusive::<u64>::HASH_AS_BYTES);
        assert!(RangeTo::<u64>::HASH_AS_BYTES);
        assert!(RangeToInclusive::<u64>::HASH_AS_BYTES);
        assert!(!Pin::<&mut u64>::HASH_AS_BYTES);
        assert!(!Poll::<u64>::HASH_AS_BYTES);
        assert!(!Duration::HASH_AS_BYTES);
    }

    #[rustversion::since(1.64)]
    #[test]
    fn test_hash_bytes_cstr() {
        assert!(!CStr::HASH_AS_BYTES);
    }

    #[rustversion::since(1.74)]
    #[test]
    fn test_hash_bytes_saturating() {
        assert!(Saturating::<u64>::HASH_AS_BYTES);
    }
}
