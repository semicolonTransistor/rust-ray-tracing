use std::{fmt::Debug, cmp::Ordering, ops::RangeBounds};
use array_macro::array;
use num::{Float, Integer};

// marker trait for scaler objects
pub trait Scaler : Copy + Clone + Debug{}
impl Scaler for f64 {}
impl Scaler for f32 {}
impl Scaler for i64 {}
impl Scaler for i32 {}
impl Scaler for i16 {}
impl Scaler for i8 {}
impl Scaler for isize{}
impl Scaler for u64 {}
impl Scaler for u32 {}
impl Scaler for u16 {}
impl Scaler for u8 {}
impl Scaler for usize{}
impl Scaler for bool {}



#[derive(Copy, Clone)]
#[derive(Debug)]
pub struct Packed<T: Scaler, const N: usize> 
(
    [T; N]
);

pub type PackedF64<const N: usize> = Packed<f64, N>;
pub type PackedF32<const N: usize> = Packed<f64, N>;
pub type PackedBool<const N: usize> = Packed<bool, N>;

impl <T, const N: usize> Packed<T, N>
where T: Scaler
{
    #[inline]
    pub fn len() -> usize {
        N
    }

    #[inline]
    pub fn inside<R, U>(&self, range: &R) -> PackedBool<N>
    where
        R: RangeBounds<U>,
        U: PartialOrd<T>,
        T: PartialOrd<U>
    {
        PackedBool::from(
            array![i => range.contains(&self[i]); N]
        )
    }

    #[inline]
    pub fn assign_masked(&mut self, values: Packed<T, N>, mask: PackedBool<N>) {
        for i in 0..N {
            if mask[i] {
                self[i] = values[i]
            }
        }
    }
}

impl <T, const N: usize> Default for Packed<T,N>
where
    T: Scaler + Default
{
    fn default() -> Self {
        Self::from(
            array![T::default(); N]
        )
    }
}

impl <T, I, const N: usize> std::ops::Index<I> for Packed<T, N>
where 
    T: Scaler,
    I: Integer,
    [T]: std::ops::Index<I>
{
    type Output = <[T] as std::ops::Index<I>>::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        &self.0[index]
    }
}

impl <T, I, const N: usize> std::ops::IndexMut<I> for Packed<T, N>
where 
    T: Scaler,
    I: Integer,
    [T]: std::ops::IndexMut<I>
{

    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl <T, const N: usize> From<[T; N]> for Packed<T, N> 
where T: Scaler
{
    #[inline]
    fn from(value: [T; N]) -> Self {
        Packed::<T, N>(
            value
        )
    }
}

impl <T, const N: usize> From<T> for Packed<T, N>
where T: Scaler
{
    #[inline]
    fn from(value: T) -> Self {
        Self::broadcast_scaler(value)
    }
}

impl <T, const N: usize> Packed<T, N>
where T: Scaler
{
    #[inline]
    pub fn broadcast_scaler(scaler: T) -> Self {
        Packed::<T, N>(
            array![scaler; N]
        )
    }
}

impl <T, U, const N: usize> std::ops::Add<Packed<U, N>> for Packed<T, N> 
where 
    T: std::ops::Add<U> + Scaler,
    U: Scaler,
    <T as std::ops::Add<U>>::Output: Scaler
{
    type Output = Packed<<T as std::ops::Add<U>>::Output, N>;

    #[inline]
    fn add(self, rhs: Packed<U, N>) -> Self::Output {
        Packed::<<T as std::ops::Add<U>>::Output, N> (
            array![i => self.0[i] + rhs.0[i]; N]
        )
    }
}

impl <T, U, const N: usize> std::ops::Add<U> for Packed<T, N> 
where
    T: std::ops::Add<U> + Scaler,
    U: Scaler,
    <T as std::ops::Add<U>>::Output: Scaler
{
    type Output = Packed<<T as std::ops::Add<U>>::Output, N>;

    #[inline]
    fn add(self, rhs: U) -> Self::Output {
        Packed::<<T as std::ops::Add<U>>::Output, N> (
            array![i => self.0[i] + rhs; N]
        )
    }
}

impl <T, U, const N: usize> std::ops::BitAnd<Packed<U, N>> for Packed<T, N> 
where 
    T: std::ops::BitAnd<U> + Scaler,
    U: Scaler,
    <T as std::ops::BitAnd<U>>::Output: Scaler
{
    type Output = Packed<<T as std::ops::BitAnd<U>>::Output, N>;

    #[inline]
    fn bitand(self, rhs: Packed<U, N>) -> Self::Output{
        Packed::<<T as std::ops::BitAnd<U>>::Output, N> (
            array![i => self.0[i] & rhs.0[i]; N]
        )
    }
}

impl <T, U, const N: usize> std::ops::BitAnd<U> for Packed<T, N> 
where
    T: std::ops::BitAnd<U> + Scaler,
    U: Scaler,
    <T as std::ops::BitAnd<U>>::Output: Scaler
{
    type Output = Packed<<T as std::ops::BitAnd<U>>::Output, N>;

    #[inline]
    fn bitand(self, rhs: U) -> Self::Output {
        Packed::<<T as std::ops::BitAnd<U>>::Output, N> (
            array![i => self.0[i] & rhs; N]
        )
    }
}

impl <T, U, const N: usize> std::ops::BitOr<Packed<U, N>> for Packed<T, N> 
where 
    T: std::ops::BitOr<U> + Scaler,
    U: Scaler,
    <T as std::ops::BitOr<U>>::Output: Scaler
{
    type Output = Packed<<T as std::ops::BitOr<U>>::Output, N>;

    #[inline]
    fn bitor(self, rhs: Packed<U, N>) -> Self::Output{
        Packed::<<T as std::ops::BitOr<U>>::Output, N> (
            array![i => self.0[i] | rhs.0[i]; N]
        )
    }
}

impl <T, U, const N: usize> std::ops::BitOr<U> for Packed<T, N> 
where
    T: std::ops::BitOr<U> + Scaler,
    U: Scaler,
    <T as std::ops::BitOr<U>>::Output: Scaler
{
    type Output = Packed<<T as std::ops::BitOr<U>>::Output, N>;

    #[inline]
    fn bitor(self, rhs: U) -> Self::Output {
        Packed::<<T as std::ops::BitOr<U>>::Output, N> (
            array![i => self.0[i] | rhs; N]
        )
    }
}

impl <T, U, const N: usize> std::ops::BitXor<Packed<U, N>> for Packed<T, N> 
where 
    T: std::ops::BitXor<U> + Scaler,
    U: Scaler,
    <T as std::ops::BitXor<U>>::Output: Scaler
{
    type Output = Packed<<T as std::ops::BitXor<U>>::Output, N>;

    #[inline]
    fn bitxor(self, rhs: Packed<U, N>) -> Self::Output{
        Packed::<<T as std::ops::BitXor<U>>::Output, N> (
            array![i => self.0[i] ^ rhs.0[i]; N]
        )
    }
}

impl <T, U, const N: usize> std::ops::BitXor<U> for Packed<T, N> 
where
    T: std::ops::BitXor<U> + Scaler,
    U: Scaler,
    <T as std::ops::BitXor<U>>::Output: Scaler
{
    type Output = Packed<<T as std::ops::BitXor<U>>::Output, N>;

    #[inline]
    fn bitxor(self, rhs: U) -> Self::Output {
        Packed::<<T as std::ops::BitXor<U>>::Output, N> (
            array![i => self.0[i] ^ rhs; N]
        )
    }
}

impl <T, U, const N: usize> std::ops::Sub<Packed<U, N>> for Packed<T, N> 
where
    T: std::ops::Sub<U> + Scaler,
    U: Scaler,
    <T as std::ops::Sub<U>>::Output: Scaler
{
    type Output = Packed<<T as std::ops::Sub<U>>::Output, N>;

    #[inline]
    fn sub(self, rhs: Packed<U, N>) -> Self::Output {
        Packed::<<T as std::ops::Sub<U>>::Output, N> (
            array![i => self.0[i] - rhs.0[i]; N]
        )
    }
}

impl <T, U, const N: usize> std::ops::Sub<U> for Packed<T, N> 
where
    T: std::ops::Sub<U> + Scaler,
    U: Scaler,
    <T as std::ops::Sub<U>>::Output: Scaler
{
    type Output = Packed<<T as std::ops::Sub<U>>::Output, N>;

    #[inline]
    fn sub(self, rhs: U) -> Self::Output {
        Packed::<<T as std::ops::Sub<U>>::Output, N> (
            array![i => self.0[i] - rhs; N]
        )
    }
}

impl <T, U, const N: usize> std::ops::Mul<Packed<U, N>> for Packed<T, N> 
where 
    T: std::ops::Mul<U> + Scaler,
    U: Scaler,
    <T as std::ops::Mul<U>>::Output: Scaler
{
    type Output = Packed<<T as std::ops::Mul<U>>::Output, N>;

    #[inline]
    fn mul(self, rhs: Packed<U, N>) -> Self::Output {
        Packed::<<T as std::ops::Mul<U>>::Output, N> (
            array![i => self.0[i] * rhs.0[i]; N]
        )
    }
}

impl <T, U, const N: usize> std::ops::Mul<U> for Packed<T, N> 
where
    T: std::ops::Mul<U> + Scaler,
    U: Scaler,
    <T as std::ops::Mul<U>>::Output: Scaler
{
    type Output = Packed<<T as std::ops::Mul<U>>::Output, N>;

    #[inline]
    fn mul(self, rhs: U) -> Self::Output {
        Packed::<<T as std::ops::Mul<U>>::Output, N> (
            array![i => self.0[i] * rhs; N]
        )
    }
}

impl <T, U, const N: usize> std::ops::Div<Packed<U, N>> for Packed<T, N> 
where
    T: std::ops::Div<U> + Scaler,
    U: Scaler,
    <T as std::ops::Div<U>>::Output: Scaler
{
    type Output = Packed<<T as std::ops::Div<U>>::Output, N>;

    #[inline]
    fn div(self, rhs: Packed<U, N>) -> Self::Output {
        Packed::<<T as std::ops::Div<U>>::Output, N> (
            array![i => self.0[i] / rhs.0[i]; N]
        )
    }
}

impl <T, U, const N: usize> std::ops::Div<U> for Packed<T, N> 
where
    T: std::ops::Div<U> + Scaler,
    U: Scaler,
    <T as std::ops::Div<U>>::Output: Scaler
{
    type Output = Packed<<T as std::ops::Div<U>>::Output, N>;

    #[inline]
    fn div(self, rhs: U) -> Self::Output {
        Packed::<<T as std::ops::Div<U>>::Output, N> (
            array![i => self.0[i] / rhs; N]
        )
    }
}

impl <T, const N: usize> std::ops::Neg for Packed<T, N> 
where
    T: std::ops::Neg + Scaler,
    <T as std::ops::Neg>::Output: Scaler
{
    type Output = Packed::<<T as std::ops::Neg>::Output, N>;

    #[inline]
    fn neg(self) -> Self::Output {
        Packed::<<T as std::ops::Neg>::Output, N> (
            array![i => -(self.0[i]); N]
        )
    }
}

impl <T, const N: usize> std::ops::Not for Packed<T, N> 
where
    T: std::ops::Not + Scaler,
    <T as std::ops::Not>::Output: Scaler
{
    type Output = Packed::<<T as std::ops::Not>::Output, N>;

    #[inline]
    fn not(self) -> Self::Output {
        Packed::<<T as std::ops::Not>::Output, N> (
            array![i => !(self.0[i]); N]
        )
    }
}

pub trait PackedPartialEq<U, const N:usize> 
where 
    U: Scaler
{
    fn eq(&self, rhs: &Packed<U, N>) -> PackedBool<N>;

    #[inline]
    fn ne(&self, rhs: &Packed<U, N>) -> PackedBool<N> {
        !self.eq(rhs)
    }
}

impl <T, U, const N:usize> PackedPartialEq<U, N> for Packed<T, N> 
where
    T: Scaler + PartialEq<U>,
    U: Scaler,
{
    #[inline]
    fn eq(&self, rhs: &Packed<U, N>) -> PackedBool<N> {
        PackedBool::from(
            array![i => self[i] == rhs[i]; N]
        )
    }
}

pub trait PackedScalerPartialEq<U, const N:usize>
where
    U: Scaler
{
    fn eq(&self, rhs: &U) -> PackedBool<N>;

    #[inline]
    fn ne(&self, rhs: &U) -> PackedBool<N> {
        !self.eq(rhs)
    }
}

impl <T, U, const N:usize> PackedScalerPartialEq<U, N> for Packed<T, N> 
where
    T: Scaler + PartialEq<U>,
    U: Scaler,
{
    #[inline]
    fn eq(&self, rhs: &U) -> PackedBool<N> {
        PackedBool::from(
            array![i => self[i] == *rhs; N]
        )
    }
}


pub trait PackedPartialOrd<U, const N:usize> 
where U: Scaler
{
    fn partial_cmp(&self, other: &Packed<U, N>) -> [Option<Ordering>; N];

    #[inline]
    fn lt(&self, other: &Packed<U, N>) -> PackedBool<N> {
        let cmp_result = self.partial_cmp(other);
        PackedBool::from(
            array![i => {
                match cmp_result[i] {
                    Some(ordering) => ordering.is_lt(),
                    None => false,
                }
            }; N]
        )
    }

    #[inline]
    fn le(&self, other: &Packed<U, N>) -> PackedBool<N> {
        let cmp_result = self.partial_cmp(other);
        PackedBool::from(
            array![i => {
                match cmp_result[i] {
                    Some(ordering) => ordering.is_le(),
                    None => false,
                }
            }; N]
        )
    }

    #[inline]
    fn gt(&self, other: &Packed<U, N>) -> PackedBool<N> {
        let cmp_result = self.partial_cmp(other);
        PackedBool::from(
            array![i => {
                match cmp_result[i] {
                    Some(ordering) => ordering.is_gt(),
                    None => false,
                }
            }; N]
        )
    }

    #[inline]
    fn ge(&self, other: &Packed<U, N>) -> PackedBool<N> {
        let cmp_result = self.partial_cmp(other);
        PackedBool::from(
            array![i => {
                match cmp_result[i] {
                    Some(ordering) => ordering.is_ge(),
                    None => false,
                }
            }; N]
        )
    }
}

impl <T, U, const N: usize> PackedPartialOrd<U, N> for Packed<T, N>
where
    T: Scaler + PartialOrd<U>,
    U: Scaler
{
    #[inline]
    fn partial_cmp(&self, other: &Packed<U, N>) -> [Option<Ordering>; N] {
        array![
            i => self[i].partial_cmp(&other[i]);
            N
        ]
    }
}

pub trait PackedScalerPartialOrd<U, const N:usize> 
where U: Scaler
{
    fn partial_cmp(&self, other: &U) -> [Option<Ordering>; N];

    #[inline]
    fn lt(&self, other: &U) -> PackedBool<N> {
        let cmp_result = self.partial_cmp(other);
        PackedBool::from(
            array![i => {
                match cmp_result[i] {
                    Some(ordering) => ordering.is_lt(),
                    None => false,
                }
            }; N]
        )
    }

    #[inline]
    fn le(&self, other: &U) -> PackedBool<N> {
        let cmp_result = self.partial_cmp(other);
        PackedBool::from(
            array![i => {
                match cmp_result[i] {
                    Some(ordering) => ordering.is_le(),
                    None => false,
                }
            }; N]
        )
    }

    #[inline]
    fn gt(&self, other: &U) -> PackedBool<N> {
        let cmp_result = self.partial_cmp(other);
        PackedBool::from(
            array![i => {
                match cmp_result[i] {
                    Some(ordering) => ordering.is_gt(),
                    None => false,
                }
            }; N]
        )
    }

    #[inline]
    fn ge(&self, other: &U) -> PackedBool<N> {
        let cmp_result = self.partial_cmp(other);
        PackedBool::from(
            array![i => {
                match cmp_result[i] {
                    Some(ordering) => ordering.is_ge(),
                    None => false,
                }
            }; N]
        )
    }
}

impl <T, U, const N: usize> PackedScalerPartialOrd<U, N> for Packed<T, N>
where
    T: Scaler + PartialOrd<U>,
    U: Scaler
{
    #[inline]
    fn partial_cmp(&self, other: &U) -> [Option<Ordering>; N] {
        array![
            i => self[i].partial_cmp(&other);
            N
        ]
    }
}

impl <T, const N: usize> Packed<T, N>
where 
    T: Scaler + std::ops::Add<T, Output = T>
{
    #[inline]
    pub fn sum(&self) -> T {
        let mut result = self[0];
        for i in 1..N{
            result = result + self[i];
        }
        result
    }
}

impl <T, const N: usize> Packed<T, N> 
where T: Scaler + Float
{
    #[inline]
    pub fn floor(&self) -> Packed<T, N> {
        Packed::from(
            array![i => self.0[i].floor(); N]
        )
    }

    #[inline]
    pub fn ceil(&self) -> Packed<T, N> {
        Packed::from(
            array![i => self.0[i].ceil(); N]
        )
    }

    #[inline]
    pub fn round(&self) -> Packed<T, N> {
        Packed::from(
            array![i => self.0[i].round(); N]
        )
    }

    #[inline]
    pub fn trunc(&self) -> Packed<T, N> {
        Packed::from(
            array![i => self.0[i].trunc(); N]
        )
    }

    #[inline]
    pub fn fract(&self) -> Packed<T, N> {
        Packed::from(
            array![i => self.0[i].fract(); N]
        )
    }

    #[inline]
    pub fn abs(&self) -> Packed<T, N> {
        Packed::from(
            array![i => self.0[i].abs(); N]
        )
    }

    #[inline]
    pub fn recip(&self) -> Packed<T, N> {
        Packed::from(
            array![i => self.0[i].recip(); N]
        )
    }

    #[inline]
    pub fn powi(&self, n:i32) -> Packed<T, N> {
        Packed::from(
            array![i => self.0[i].powi(n); N]
        )
    } 

    #[inline]
    pub fn powf(&self, n: T) -> Packed<T, N> {
        Packed::from(
            array![i => self.0[i].powf(n); N]
        )
    }

    #[inline]
    pub fn sqrt(&self) -> Packed<T, N> {
        Packed::from(
            array![i => self.0[i].sqrt(); N]
        )
    }

    #[inline]
    pub fn exp(&self) -> Packed<T, N> {
        Packed::from(
            array![i => self.0[i].exp(); N]
        )
    }

    #[inline]
    pub fn exp2(&self) -> Packed<T, N> {
        Packed::from(
            array![i => self.0[i].exp2(); N]
        )
    }

    #[inline]
    pub fn ln(&self) -> Packed<T, N> {
        Packed::from(
            array![i => self.0[i].ln(); N]
        )
    }

    #[inline]
    pub fn log(&self, base: T) -> Packed<T, N> {
        Packed::from(
            array![i => self.0[i].log(base); N]
        )
    }

    #[inline]
    pub fn log2(&self) -> Packed<T, N> {
        Packed::from(
            array![i => self.0[i].log2(); N]
        )
    }

    #[inline]
    pub fn log10(&self) -> Packed<T, N> {
        Packed::from(
            array![i => self.0[i].log10(); N]
        )
    }

    #[inline]
    pub fn elementwise_max(&self, other: Packed<T,N>) -> Packed<T, N> {
        Packed::from(
            array![i => self.0[i].max(other.0[i]); N]
        )
    }

    #[inline]
    pub fn elementwise_min(&self, other: Packed<T,N>) -> Packed<T, N> {
        Packed::from(
            array![i => self.0[i].min(other.0[i]); N]
        )
    }

    #[inline]
    pub fn sin(&self) -> Packed<T, N> {
        Packed::from(
            array![i => self.0[i].sin(); N]
        )
    }

    #[inline]
    pub fn cos(&self) -> Packed<T, N> {
        Packed::from(
            array![i => self.0[i].cos(); N]
        )
    }

}

impl <const N:usize> PackedBool<N> {
    #[inline]
    pub fn all(&self) -> bool {
        self.0.iter().all(|b| {*b})
    }

    #[inline]
    pub fn any(&self) -> bool {
        self.0.iter().any(|b| {*b})
    }

    #[inline]
    pub fn set(&mut self, mask: &PackedBool<N>) {
        *self = *self | *mask;
    }

    #[inline]
    pub fn clear(&mut self, mask: &PackedBool<N>) {
        *self = *self & !(*mask);
    }
}