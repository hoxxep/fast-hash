//! [`PortableHash`] implementations for standard library types.

use std::{
    // explicitly omitted: ffi::{OsStr, OsString},
    // explicitly omitted: fs::FileType,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    // explicitly omitted: path::{Component, Path, PathBuf, Prefix, PrefixComponent},
    // explicitly omitted: thread::ThreadId,
    // TODO(stabilisation): should we even hash system time types?
    time::{Instant, SystemTime},
};
use crate::{PortableHash, PortableHasher};

impl PortableHash for IpAddr {
    #[inline]
    fn portable_hash<H: PortableHasher>(&self, state: &mut H) {
        match self {
            IpAddr::V4(addr) => addr.portable_hash(state),
            IpAddr::V6(addr) => addr.portable_hash(state),
        }
    }
}

impl PortableHash for Ipv4Addr {
    #[inline]
    fn portable_hash<H: PortableHasher>(&self, state: &mut H) {
        state.write(&self.octets());
    }
}

impl PortableHash for Ipv6Addr {
    #[inline]
    fn portable_hash<H: PortableHasher>(&self, state: &mut H) {
        state.write(&self.octets());
    }
}

impl PortableHash for SocketAddr {
    #[inline]
    fn portable_hash<H: PortableHasher>(&self, state: &mut H) {
        match self {
            SocketAddr::V4(addr) => addr.portable_hash(state),
            SocketAddr::V6(addr) => addr.portable_hash(state),
        }
    }
}

impl PortableHash for SocketAddrV4 {
    #[inline]
    fn portable_hash<H: PortableHasher>(&self, state: &mut H) {
        self.ip().portable_hash(state);
        state.write_u16(self.port());
    }
}

impl PortableHash for SocketAddrV6 {
    #[inline]
    fn portable_hash<H: PortableHasher>(&self, state: &mut H) {
        self.ip().portable_hash(state);
        state.write_u16(self.port());
        state.write_u32(self.flowinfo());
        state.write_u32(self.scope_id());
    }
}

impl PortableHash for Instant {
    #[inline]
    fn portable_hash<H: PortableHasher>(&self, state: &mut H) {
        self.elapsed().portable_hash(state);
    }
}

impl PortableHash for SystemTime {
    #[inline]
    fn portable_hash<H: PortableHasher>(&self, state: &mut H) {
        // TODO(stabilisation): hashing of negative SystemTime is bad!
        match self.elapsed() {
            Ok(duration) => duration.portable_hash(state),
            Err(_) => {
                // If the SystemTime is before UNIX_EPOCH, we can use a sentinel value.
                state.write_u64(0);
                state.write_u32(0);
            }
        }
    }
}
