//! The Argus frame wire format: one UDP datagram per 50 ms bin from the
//! board, as defined by `argus_wire.h` in argus_core (version 3, 594 bytes).
//!
//! Every field is little-endian and packed (no padding):
//!
//! | offset | field         | type      |
//! |--------|---------------|-----------|
//! | 0      | magic         | u32       |
//! | 4      | sample        | u32       |
//! | 8      | t             | f32       |
//! | 12     | version       | u8        |
//! | 13     | reserved      | u8        |
//! | 14     | channel_count | u16       |
//! | 16     | channels      | [u16; 96] |
//! | 208    | power         | [u32; 96] |
//! | 592    | crc           | u16       |

pub struct Frame {}
