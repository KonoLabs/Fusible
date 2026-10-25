//! A small non-cryptographic 128-bit hash for cache keys.

/// Two FNV-1a lanes, 128 bits in total.
pub(super) struct Hasher(u64, u64);

impl Hasher {
    pub(super) fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325, 0x8422_2325_cbf2_9ce4)
    }

    pub(super) fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 = (self.0 ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3);
            self.1 = (self.1 ^ u64::from(byte.rotate_left(3))).wrapping_mul(0x0000_0100_0000_01b3);
        }
        self.write_len(bytes.len());
    }

    fn write_len(&mut self, len: usize) {
        self.1 = self.1.rotate_left(7) ^ len as u64;
    }

    pub(super) fn finish(&self) -> String {
        format!("{:016x}{:016x}", self.0, self.1)
    }
}
