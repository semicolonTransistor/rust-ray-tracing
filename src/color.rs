use crate::{toml_utils::to_float, simd_util::SimdPermute};

use std::simd::{Simd, Mask, LaneCount, SupportedLaneCount, cmp::SimdPartialOrd, SimdElement};
use array_macro::array;
use crate::simd_util::masked_assign;

#[derive(Debug)]
#[derive(Clone, Copy)]
pub struct Color {
    red: f64,
    green: f64,
    blue: f64
}

impl Color {
    pub fn new(red: f64, green: f64, blue: f64) -> Color {
        Color {
            red: red,
            green: green,
            blue: blue,
        }
    }

    pub fn black() -> Color {
        Color {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
        }
    }

    pub fn white() -> Color {
        Color {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
        }
    }

    // pub fn to_ppm(&self, max_pixel_value: u32) -> String {
    //     let result = self.to_array(max_pixel_value);
    //     format!("{} {} {}\n", result[0], result[1], result[2])
    // }

    // pub fn to_array(&self, max_pixel_value: u32) -> [u32; 3] 
    // {
    //     let scale_factor = (max_pixel_value as f64 ) + 1.0 - 0.001;
    //     let ir = (self.red.sqrt() * scale_factor) as u32;
    //     let ig = (self.green.sqrt() * scale_factor) as u32;
    //     let ib = (self.blue.sqrt() * scale_factor) as u32;

    //     return [ir, ig, ib];
    // }

    pub fn to_u8_array(&self) -> [u8; 3]{
        assert!(self.red <= 2.0, "red should be less than 1.0, but got {}", self.red);
        assert!(self.green <= 2.0, "green should be less than 1.0, but got {}", self.green);
        assert!(self.blue <= 2.0, "blue should be less than 1.0, but got {}", self.blue);
        let scale_factor = 255.999;
        let ir = (self.red.sqrt() * scale_factor) as u8;
        let ig = (self.green.sqrt() * scale_factor) as u8;
        let ib = (self.blue.sqrt() * scale_factor) as u8;

        return [ir, ig, ib];
    }

    pub fn average<F>(iter: F) -> Color
    where F: IntoIterator<Item=Color> 
    {
        let mut red = 0.0;
        let mut green = 0.0;
        let mut blue = 0.0;
        let mut count: i32 = 0;

        for color in iter {
            count += 1;
            red += color.red;
            green += color.green;
            blue += color.blue;
        }

        Color {
            red: red / (count as f64),
            green: green / (count as f64),
            blue: blue / (count as f64),
        }
    }

    // pub fn mix(&self, other: &Color, other_ratio: f64) -> Color {
    //     debug_assert!((0.0..=1.0).contains(&other_ratio));
    //     let self_ratio = 1.0 - other_ratio;

    //     Color { 
    //         red: self_ratio * self.red + other_ratio * other.red,
    //         green: self_ratio * self.green + other_ratio * other.green,
    //         blue: self_ratio * self.blue+ other_ratio * other.blue 
    //     }
    // }

    pub fn from_toml(value :&toml::Value) -> Option<Color> {
        match value.as_table() {
            Some(table) => {
                let red = to_float(&table["red"]).unwrap_or(0.0);
                let green = to_float(&table["green"]).unwrap_or(0.0);
                let blue = to_float(&table["blue"]).unwrap_or(0.0);

                Some(Color::new(red, green, blue))
            },
            None => match value.as_array() {
                Some(array) => {
                    assert!(array.len() >= 3);
                    let red = to_float(&array[0]).unwrap();
                    let green = to_float(&array[1]).unwrap();
                    let blue = to_float(&array[2]).unwrap();

                    Some(Color::new(red, green, blue))
                },
                None => None,
            },
        }
        

        
    }

    
}

impl std::ops::Mul<Color> for Color {
    type Output = Color;

    #[inline]
    fn mul(self, rhs: Color) -> Self::Output {
        Color{
            red: self.red * rhs.red,
            green: self.green * rhs.green,
            blue: self.blue * rhs.blue,
        }
    }
}

impl std::ops::Mul<f64> for Color {
    type Output = Color;

    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        Color{
            red: self.red * rhs,
            green: self.green * rhs,
            blue: self.blue * rhs,
        }
    }
}

impl std::ops::Div<f64> for Color {
    type Output = Color;

    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        Color{
            red: self.red / rhs,
            green: self.green / rhs,
            blue: self.blue / rhs,
        }
    }
}


impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.2}, {:.2}, {:.2})", self.red, self.green, self.blue)
    }
}

#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct PackedColor<const N: usize> 
where LaneCount<N>: SupportedLaneCount
{
    red: Simd<f64, N>,
    green: Simd<f64, N>,
    blue: Simd<f64, N>
}

impl <const N: usize> PackedColor<N> 
where LaneCount<N>: SupportedLaneCount
{
    #[inline]
    pub fn broadcast_scaler(color: Color) -> PackedColor<N> {
        Self::splat(color)
    }

    #[inline]
    pub fn splat(color: Color) -> PackedColor<N> {
        PackedColor {
            red: Simd::splat(color.red),
            green: Simd::splat(color.green),
            blue: Simd::splat(color.blue),
        }
    }

    #[inline]
    pub fn assign_masked(&mut self, colors: PackedColor<N>, mask: Mask<<f64 as SimdElement>::Mask, N>) {
        
        masked_assign(&mut self.red, colors.red, mask);
        masked_assign(&mut self.green, colors.green, mask);
        masked_assign(&mut self.blue, colors.blue, mask);
    }

    #[inline]
    pub fn at(&self, index: usize) -> Color {
        Color {
            red: self.red[index],
            green: self.green[index],
            blue: self.blue[index]
        }
    }

    #[inline]
    pub fn update(&mut self, value: Color, index: usize) {
        self.red[index] = value.red;
        self.green[index] = value.green;
        self.blue[index] = value.blue;
    }

    #[inline]
    pub fn sum(&self) -> Color {
        Color {
            red: self.red.as_array().iter().sum(),
            green: self.green.as_array().iter().sum(),
            blue: self.blue.as_array().iter().sum()
        }
    }

    #[inline]
    pub fn check(&self) {
        assert!((self.red.simd_lt(Simd::splat(1.01))).all(), "RED Expect <= 1.0, got{:?}", self.red);
        assert!((self.green.simd_lt(Simd::splat(1.01))).all(), "GREEN Expect <= 1.0, got{:?}", self.green);
        assert!((self.blue.simd_lt(Simd::splat(1.01))).all(), "BLUE Expect <= 1.0, got{:?}", self.blue);
    }
}

impl <const N: usize> std::ops::Mul<PackedColor<N>> for PackedColor<N> 
where LaneCount<N>: SupportedLaneCount
{
    type Output = PackedColor<N>;
    #[inline]
    fn mul(self, rhs: PackedColor<N>) -> Self::Output {
        PackedColor{
            red: self.red * rhs.red,
            green: self.green * rhs.green,
            blue: self.blue * rhs.blue,
        }
    }
}

impl <const N: usize> std::ops::Mul<Simd<f64, N>> for PackedColor<N> 
where LaneCount<N>: SupportedLaneCount
{
    type Output = PackedColor<N>;
    #[inline]
    fn mul(self, rhs: Simd<f64, N>) -> Self::Output {
        PackedColor{
            red: self.red * rhs,
            green: self.green * rhs,
            blue: self.blue * rhs,
        }
    }
}

impl <const N: usize> std::ops::Mul<PackedColor<N>> for Simd<f64, N> 
where LaneCount<N>: SupportedLaneCount
{
    type Output = PackedColor<N>;
    #[inline]
    fn mul(self, rhs: PackedColor<N>) -> Self::Output {
        PackedColor{
            red: self * rhs.red,
            green: self * rhs.green,
            blue: self * rhs.blue,
        }
    }
}

impl <const N: usize> std::ops::Add<PackedColor<N>> for PackedColor<N> 
where LaneCount<N>: SupportedLaneCount
{
    type Output = PackedColor<N>;
    #[inline]
    fn add(self, rhs: PackedColor<N>) -> Self::Output {
        PackedColor{
            red: self.red + rhs.red,
            green: self.green + rhs.green,
            blue: self.blue + rhs.blue,
        }
    }
}

impl <const N: usize> SimdPermute<N> for &mut [PackedColor<N>] 
where 
    LaneCount<N>: SupportedLaneCount
{
    #[inline]
    fn permute(&mut self, tmp_buffer: Self, chunk_indices: &[Simd<usize, N>], lane_indices: &[Simd<usize, N>]) {
            unsafe {
                tmp_buffer.copy_from_slice(self);
                let temp_as_slice: &[f64] = std::slice::from_raw_parts(std::mem::transmute(tmp_buffer.as_ptr()), self.len() * N * 3);
                
                for i in 0..self.len() {
                    self[i].red = Simd::gather_or_default(temp_as_slice, chunk_indices[i] * Simd::splat(N * 3) + lane_indices[i]);
                    self[i].green = Simd::gather_or_default(temp_as_slice, chunk_indices[i] * Simd::splat(N * 3) + Simd::splat(N) + lane_indices[i]);
                    self[i].blue = Simd::gather_or_default(temp_as_slice, chunk_indices[i] * Simd::splat(N * 3) + Simd::splat(2 * N) + lane_indices[i]);
                }
            }
    }
}