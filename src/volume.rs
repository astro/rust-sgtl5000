use core::ops::Sub;

#[derive(Debug)]
pub struct Volume {
    left: u8,
    right: u8,
}

impl From<(u8, u8)> for Volume {
    fn from((left, right): (u8, u8)) -> Self {
        Volume { left, right }
    }
}

impl From<u8> for Volume {
    fn from(vol: u8) -> Self {
        Volume { left: vol, right: vol }
    }
}

impl Volume {
    pub fn to_range(&self, quiet: u8, full: u8) -> (u8, u8) {
        let left = volume_to_range(self.left, quiet, full);
        let right = volume_to_range(self.right, quiet, full);
        (left, right)
    }
}

fn volume_to_range(v: u8, quiet: u8, full: u8) -> u8 {
    let quiet = quiet as i16;
    let full = full as i16;
    let r = quiet + (v as i16) * (full - quiet);
    if quiet < full {
        r.min(full).max(quiet) as u8
    } else {
        r.min(quiet).max(full) as u8
    }
}
