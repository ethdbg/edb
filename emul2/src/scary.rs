//! unsafe operations used in Emulator
//! Use of these functions requires a comment that describes why the function needs to be used, and
//! why it won't nuke the program, computer, or universe when it runs

/// non-scalar type-casts
crate mod non_scalar_typecast {

    /// convert 8 bytes into a unsigned 64-bit integer
    crate unsafe fn to_u64(val: [u8; 8]) -> u64 {
        std::mem::transmute::<[u8; 8], u64>(val)
    }

    /// convert a ethereum_types::H256 to an array of 4 unsigned 64bit integers
    crate unsafe fn h256_to_u256(val: ethereum_types::H256) -> [u64; 4] {
        std::mem::transmute::<[u8; 32], [u64; 4] >(val.0)
    }
}

