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

/// "ARGS" read as a little-endian u32 (ARGUS_FRAME_MAGIC).
pub const FRAME_MAGIC: u32 = 0x4152_4753;
/// The only wire version this receiver accepts; v3 added `power`.
pub const FRAME_VERSION: u8 = 3;
/// Channels in every frame (ARGUS_MAX_CHANNELS).
pub const MAX_CHANNELS: usize = 96;
/// The UDP port the board sends frames to (ARGUS_UDP_PORT).
pub const UDP_PORT: u16 = 5005;

// Byte offsets of each field, from the table above.
const MAGIC_AT: usize = 0;
const SAMPLE_AT: usize = 4;
const T_AT: usize = 8;
const VERSION_AT: usize = 12;
const CHANNEL_COUNT_AT: usize = 14;
const CHANNELS_AT: usize = 16;
const POWER_AT: usize = CHANNELS_AT + 2 * MAX_CHANNELS;
const CRC_AT: usize = POWER_AT + 4 * MAX_CHANNELS;

/// Size of a version 3 frame in bytes.
pub const FRAME_LEN: usize = CRC_AT + 2;

/// One frame, decoded.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    /// Sample index on the board when the bin closed.
    pub sample: u32,
    /// Seconds since the firmware started.
    pub t: f32,
    /// Channels in use, clamped to MAX_CHANNELS.
    pub channel_count: u16,
    /// Threshold crossings per channel in this bin.
    pub channels: [u16; MAX_CHANNELS],
    /// Mean-square spike-band power per channel, in ADC code squared.
    pub power: [u32; MAX_CHANNELS],
}

/// Why a datagram was rejected. The receiver keeps one counter per variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameError {
    /// Too short to hold a header, or not exactly FRAME_LEN bytes.
    BadSize,
    /// Not an Argus frame at all.
    BadMagic,
    /// An Argus frame from firmware on another wire version.
    BadVersion,
    /// The right shape, but corrupted between the board and here.
    BadCrc,
}

fn read_u16(buf: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([buf[at], buf[at + 1]])
}

fn read_u32(buf: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([buf[at], buf[at + 1], buf[at + 2], buf[at + 3]])
}

const fn build_crc_table() -> [u16; 256] {
    let mut table = [0u16; 256];
    let mut i = 0;

    while i < 256 {
        let mut crc = (i as u16) << 8;
        let mut bit = 0;
        while bit < 8 {
            crc = if crc & 0x8000 != 0 {
                (crc << 1) ^ 0x1021
            } else {
                crc << 1
            };
            bit += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

/// One table entry per byte value, so the CRC costs one lookup per byte.
/// Built at compile time from the bit-by-bit definition, so it cannot drift
/// from it; the tests check it against the table in argus_wire.h.
const CRC_TABLE: [u16; 256] = build_crc_table();

/// Continue a CRC over more bytes, for data that is not contiguous.
pub fn crc16_ccitt_update(mut crc: u16, data: &[u8]) -> u16 {
    for &byte in data {
        let index = usize::from((crc >> 8) as u8 ^ byte);
        crc = (crc << 8) ^ CRC_TABLE[index];
    }
    crc
}

/// CRC-16/CCITT-FALSE (poly 0x1021, init 0xFFFF) over `data`.
pub fn crc16_ccitt(data: &[u8]) -> u16 {
    crc16_ccitt_update(0xFFFF, data)
}

/// Parse one datagram into a Frame.
///
/// The checks run in the same order as argus_sensors' C++ parser. Magic
/// and version come before the length check, so a frame from older firmware
/// counts as a version mismatch rather than a wrong size: every version bump
/// so far has changed the size.
pub fn parse_frame(buf: &[u8]) -> Result<Frame, FrameError> {
    if buf.len() < VERSION_AT + 1 {
        return Err(FrameError::BadSize);
    }
    if read_u32(buf, MAGIC_AT) != FRAME_MAGIC {
        return Err(FrameError::BadMagic);
    }
    if buf[VERSION_AT] != FRAME_VERSION {
        return Err(FrameError::BadVersion);
    }
    if buf.len() != FRAME_LEN {
        return Err(FrameError::BadSize);
    }
    if read_u16(buf, CRC_AT) != crc16_ccitt(&buf[..CRC_AT]) {
        return Err(FrameError::BadCrc);
    }

    Ok(Frame {
        sample: read_u32(buf, SAMPLE_AT),
        t: f32::from_bits(read_u32(buf, T_AT)),
        channel_count: read_u16(buf, CHANNEL_COUNT_AT).min(MAX_CHANNELS as u16),
        channels: std::array::from_fn(|i| read_u16(buf, CHANNELS_AT + 2 * i)),
        power: std::array::from_fn(|i| read_u32(buf, POWER_AT + 4 * i)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The frame from argus_sensors' C++ test, byte for byte.
    fn make_frame() -> Vec<u8> {
        let mut buf = Vec::with_capacity(FRAME_LEN);
        buf.extend_from_slice(&FRAME_MAGIC.to_le_bytes());
        buf.extend_from_slice(&1234u32.to_le_bytes());
        buf.extend_from_slice(&61.7f32.to_le_bytes());
        buf.push(FRAME_VERSION);
        buf.push(0); // reserved
        buf.extend_from_slice(&(MAX_CHANNELS as u16).to_le_bytes());
        for i in 0..MAX_CHANNELS {
            buf.extend_from_slice(&((i % 7) as u16).to_le_bytes());
        }
        for i in 0..MAX_CHANNELS {
            buf.extend_from_slice(&(100_000 * i as u32 + 3914).to_le_bytes());
        }
        let crc = crc16_ccitt(&buf);
        buf.extend_from_slice(&crc.to_le_bytes());
        buf
    }

    /// Recompute the CRC after a test has changed a field on purpose.
    fn fix_crc(buf: &mut [u8]) {
        let crc = crc16_ccitt(&buf[..CRC_AT]);
        buf[CRC_AT..].copy_from_slice(&crc.to_le_bytes());
    }

    #[test]
    fn crc_matches_the_standard_check_value() {
        assert_eq!(crc16_ccitt(b"123456789"), 0x29B1);
    }

    #[test]
    fn crc_table_matches_argus_wire_h() {
        assert_eq!(CRC_TABLE[1], 0x1021);
        assert_eq!(CRC_TABLE[128], 0x9188);
        assert_eq!(CRC_TABLE[255], 0x1EF0);
    }

    #[test]
    fn incremental_crc_matches_one_shot() {
        let data = b"123456789";
        let crc = crc16_ccitt_update(crc16_ccitt_update(0xFFFF, &data[..4]), &data[4..]);
        assert_eq!(crc, crc16_ccitt(data));
    }

    #[test]
    fn test_frame_matches_the_c_header() {
        // 0x8260 is what argus_wire.h's crc16_ccitt() gives for this frame,
        // so this pins the field layout as well as the CRC.
        let buf = make_frame();
        assert_eq!(buf.len(), 594);
        assert_eq!(read_u16(&buf, CRC_AT), 0x8260);
    }

    #[test]
    fn accepts_current_version_and_copies_power() {
        let frame = parse_frame(&make_frame()).expect("a valid frame");
        assert_eq!(frame.sample, 1234);
        assert_eq!(frame.t, 61.7);
        assert_eq!(frame.channel_count, 96);
        for (i, (&count, &power)) in frame.channels.iter().zip(&frame.power).enumerate() {
            assert_eq!(count, (i % 7) as u16, "channel {i}");
            assert_eq!(power, 100_000 * i as u32 + 3914, "channel {i}");
        }
    }

    #[test]
    fn rejects_version_2_as_bad_version() {
        // A v2 frame: the v3 header and counts, then the CRC, no power.
        let mut buf = make_frame()[..POWER_AT].to_vec();
        buf[VERSION_AT] = 2;
        let crc = crc16_ccitt(&buf);
        buf.extend_from_slice(&crc.to_le_bytes());
        assert_eq!(buf.len(), 210);
        assert_eq!(parse_frame(&buf), Err(FrameError::BadVersion));
    }

    #[test]
    fn rejects_version_2_even_at_current_size() {
        let mut buf = make_frame();
        buf[VERSION_AT] = 2;
        assert_eq!(parse_frame(&buf), Err(FrameError::BadVersion));
    }

    #[test]
    fn rejects_bad_magic() {
        let mut buf = make_frame();
        buf[0] ^= 0xFF;
        assert_eq!(parse_frame(&buf), Err(FrameError::BadMagic));
    }

    #[test]
    fn rejects_wrong_length() {
        let mut buf = make_frame();
        assert_eq!(parse_frame(&buf[..FRAME_LEN - 1]), Err(FrameError::BadSize));
        assert_eq!(parse_frame(&buf[..4]), Err(FrameError::BadSize));
        buf.push(0);
        assert_eq!(parse_frame(&buf), Err(FrameError::BadSize));
    }

    #[test]
    fn rejects_corrupt_power() {
        let mut buf = make_frame();
        buf[POWER_AT + 17] ^= 0x01;
        assert_eq!(parse_frame(&buf), Err(FrameError::BadCrc));
    }

    #[test]
    fn clamps_channel_count() {
        let mut buf = make_frame();
        buf[CHANNEL_COUNT_AT..CHANNELS_AT].copy_from_slice(&200u16.to_le_bytes());
        fix_crc(&mut buf);
        let frame = parse_frame(&buf).expect("a valid frame");
        assert_eq!(frame.channel_count, 96);
    }
}
