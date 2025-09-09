use crate::{make_error_result, Error};

#[derive(Clone, Copy)]
pub struct Color {
    pub linear: [f32; 3],
}

impl Color {
    pub fn from_linear(r: f32, g: f32, b: f32) -> Self {
        Self { linear: [r, g, b] }
    }

    pub fn from_srgb(r: u8, g: u8, b: u8) -> Self {
        Self {
            linear: [
                Self::from_srgb_single(r),
                Self::from_srgb_single(g),
                Self::from_srgb_single(b),
            ],
        }
    }

    pub fn from_hex<H>(hex: H) -> Result<Self, Error>
    where
        H: AsRef<str>,
    {
        let s = hex.as_ref().trim_start_matches('#');
        if s.len() != 6 {
            return make_error_result("Hex color must be exactly 6 digits", None);
        }
        let r = u8::from_str_radix(&s[0..2], 16);
        let g = u8::from_str_radix(&s[2..4], 16);
        let b = u8::from_str_radix(&s[4..6], 16);

        let r_linear = match r {
            Ok(v) => Self::from_srgb_single(v),
            Err(_) => {
                return make_error_result("Could not convert string to integer in HEX color", None)
            }
        };

        let g_linear = match g {
            Ok(v) => Self::from_srgb_single(v),
            Err(_) => {
                return make_error_result("Could not convert string to integer in HEX color", None)
            }
        };

        let b_linear = match b {
            Ok(v) => Self::from_srgb_single(v),
            Err(_) => {
                return make_error_result("Could not convert string to integer in HEX color", None)
            }
        };

        Ok(Self {
            linear: [r_linear, g_linear, b_linear],
        })
    }

    fn from_srgb_single(c: u8) -> f32 {
        let fc = c as f32 / 255.0;
        if fc <= 0.04045 {
            fc / 12.92
        } else {
            ((fc + 0.055) / 1.055).powf(2.4)
        }
    }
}
