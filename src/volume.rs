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
    let quiet = quiet as i32;
    let full = full as i32;
    let r = quiet + (v as i32).saturating_mul(full - quiet) / 255;
    if quiet < full {
        r.min(full).max(quiet) as u8
    } else {
        r.min(quiet).max(full) as u8
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_dac_vol_range() {
        fn vol_to_dac<V: Into<Volume>>(v: V) -> u8 {
            let volume = v.into();
            let (left, right) = volume.to_range(0xFC, 0x3C);
            assert_eq!(left, right);
            left
        }
        assert_eq!(0x3C, vol_to_dac(255));
        assert_eq!(0xFC, vol_to_dac(0));
    }

    #[test]
    fn test_lineout_vol_range() {
        fn vol_to_lineout<V: Into<Volume>>(v: V) -> u8 {
            let volume = v.into();
            let (left, _right) = volume.to_range(0, 0x1F);
            left
        }
        assert_eq!(0x1F, vol_to_lineout(255));
        assert_eq!(0x00, vol_to_lineout(0));
    }

    #[test]
    fn test_hp_vol_range() {
        fn vol_to_hp<V: Into<Volume>>(v: V) -> u8 {
            let volume = v.into();
            let (left, _right) = volume.to_range(0x7F, 0);
            left
        }
        assert_eq!(0, vol_to_hp(255));
        assert_eq!(0x7F, vol_to_hp(0));
    }
}
