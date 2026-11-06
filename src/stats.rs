use std::fmt;

use crate::wire::{Frame, FrameError};

/// How many datagrams were accepted, and how many were
/// regected for each reason, since the receiver started.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FrameCounts {
    pub ok: u64,
    pub bad_size: u64,
    pub bad_magic: u64,
    pub bad_version: u64,
    pub bad_crc: u64,
}

impl FrameCounts {}

impl fmt::Display for FrameCounts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "frames ok={} size={} magic={} ver={} crc={}",
            self.ok, self.bad_size, self.bad_magic, self.bad_version, self.bad_crc
        )
    }
}

#[cfg(test)]
mod tests {}
