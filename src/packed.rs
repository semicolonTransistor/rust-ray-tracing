use std::{fmt::Debug, cmp::Ordering, ops::RangeBounds};
use array_macro::array;
use num::{Float, Unsigned, Integer};

// marker trait for scaler objects
pub trait Scaler : Copy + Clone + Debug{
    type MaskType: Mask + Scaler;
}
impl Scaler for f64   { type MaskType = u64;}
impl Scaler for f32   { type MaskType = u32;}
impl Scaler for i64   { type MaskType = u64;}
impl Scaler for i32   { type MaskType = u32;}
impl Scaler for i16   { type MaskType = u16;}
impl Scaler for i8    { type MaskType = u8;}
impl Scaler for isize { type MaskType = usize;}
impl Scaler for u64   { type MaskType = u64;}
impl Scaler for u32   { type MaskType = u32;}
impl Scaler for u16   { type MaskType = u16;}
impl Scaler for u8    { type MaskType = u8;}
impl Scaler for usize { type MaskType = usize;}
impl Scaler for bool  { type MaskType = u8;}

pub trait Mask: 
    Integer + Unsigned + Scaler + Ord + std::ops::Not<Output = Self> + 
    std::ops::BitAnd<Self, Output = Self> + std::ops::BitOr<Self, Output = Self> + 
    std::ops::BitXor<Self, Output = Self> 
{
    const CLEAR_VALUE: Self;
    const SET_VALUE: Self;
    const TRUE_MASK: Self;

    #[inline]
    fn mask_from_bool(b: bool) -> Self {
        if b {
            Self::SET_VALUE
        } else {
            Self::CLEAR_VALUE
        }
    }

    #[inline]
    fn to_bool(&self) -> bool {
        !((*self & Self::TRUE_MASK).is_zero())
    }
}

impl Mask for usize {
    const CLEAR_VALUE: Self = 0;
    const SET_VALUE: Self = usize::MAX;
    const TRUE_MASK: Self = usize::MAX & !(usize::MAX >> 1);
}

impl Mask for u64 {
    const CLEAR_VALUE: Self = 0;
    const SET_VALUE: Self = u64::MAX;
    const TRUE_MASK: Self = u64::MAX & !(u64::MAX >> 1);
}

impl Mask for u32 {
    const CLEAR_VALUE: Self = 0;
    const SET_VALUE: Self = u32::MAX;
    const TRUE_MASK: Self = u32::MAX & !(u32::MAX >> 1);
}

impl Mask for u16 {
    const CLEAR_VALUE: Self = 0;
    const SET_VALUE: Self = u16::MAX;
    const TRUE_MASK: Self = u16::MAX & !(u16::MAX >> 1);
}

impl Mask for u8 {
    const CLEAR_VALUE: Self = 0;
    const SET_VALUE: Self = u8::MAX;
    const TRUE_MASK: Self = u8::MAX & !(u8::MAX >> 1);
}


#[derive(Copy, Clone)]
#[derive(Debug)]
pub struct Packed<T: Scaler, const N: usize> 
(
    [T; N]
);

pub type PackedF64<const N: usize> = Packed<f64, N>;
pub type PackedF32<const N: usize> = Packed<f32, N>;
pub type PackedF64Mask<const N: usize> = Packed<<f64 as Scaler>::MaskType, N>;
pub type PackedF32Mask<const N: usize> = Packed<<f32 as Scaler>::MaskType, N>;
// pub type PackedBool<const N: usize> = Packed<bool, N>;

impl <T, const N: usize> Packed<T, N>
where T: Scaler
{
    #[inline]
    pub fn len() -> usize {
        N
    }

    #[inline]
    pub fn inside<R, U, M>(&self, range: &R) -> Packed<M, N>
    where
        R: RangeBounds<U>,
        U: PartialOrd<T>,
        M: Scaler + Mask,
        T: PartialOrd<U> + Scaler<MaskType = M>,
    {
        Packed::<M, N>::from(
            array![i => M::mask_from_bool(range.contains(&self[i])); N]
        )
    }

    #[inline]
    pub fn assign_masked<M>(&mut self, values: Packed<T, N>, mask: Packed<M, N>) 
    where 
        T: Scaler<MaskType = M>,
        M: Mask
    {
        for i in 0..N {
            if mask[i].to_bool() {
                self[i] = values[i]
            }
        }
    }

    #[inline]
    pub fn select_masked<M>(&self, values: Packed<T, N>, mask: Packed<M, N>) -> Packed<T, N> 
    where 
        T: Scaler<MaskType = M>,
        M: Mask
    {
        Packed::from(
            array![i => if mask[i].to_bool() {
                values[i]
            } else {
                self[i]
            }; N]
        )
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

pub trait PackedPartialEq<U, M, const N:usize> 
where 
    U: Scaler<MaskType = M>,
    M: Mask
{
    fn eq(&self, rhs: &Packed<U, N>) -> Packed<M, N>;

    #[inline]
    fn ne(&self, rhs: &Packed<U, N>) -> Packed<M, N> {
        !self.eq(rhs)
    }
}

impl <T, U, M, const N:usize> PackedPartialEq<U, M, N> for Packed<T, N> 
where
    T: Scaler + PartialEq<U>,
    U: Scaler<MaskType = M>,
    M: Mask
{
    #[inline]
    fn eq(&self, rhs: &Packed<U, N>) -> Packed<M, N> {
        Packed::from(
            array![i => M::mask_from_bool(self[i] == rhs[i]); N]
        )
    }
}

pub trait PackedScalerPartialEq<U, M, const N:usize>
where
    U: Scaler<MaskType = M>,
    M: Mask
{
    fn eq(&self, rhs: &U) -> Packed<M, N>;

    #[inline]
    fn ne(&self, rhs: &U) -> Packed<M, N> {
        !self.eq(rhs)
    }
}

impl <T, U, M, const N:usize> PackedScalerPartialEq<U, M, N> for Packed<T, N> 
where
    T: Scaler + PartialEq<U>,
    U: Scaler<MaskType = M>,
    M: Mask,
{
    #[inline]
    fn eq(&self, rhs: &U) -> Packed<M, N> {
        Packed::from(
            array![i => M::mask_from_bool(self[i] == *rhs); N]
        )
    }
}


pub trait PackedPartialOrd<U, M ,const N:usize> 
where 
    U: Scaler<MaskType = M>,
    M: Mask,
{
    fn partial_cmp(&self, other: &Packed<U, N>) -> [Option<Ordering>; N];

    #[inline]
    fn lt(&self, other: &Packed<U, N>) -> Packed<M, N> {
        let cmp_result = self.partial_cmp(other);
        Packed::from(
            array![i => {
                match cmp_result[i] {
                    Some(ordering) => M::mask_from_bool(ordering.is_lt()),
                    None => M::mask_from_bool(false),
                }
            }; N]
        )
    }

    #[inline]
    fn le(&self, other: &Packed<U, N>) -> Packed<M, N> {
        let cmp_result = self.partial_cmp(other);
        Packed::from(
            array![i => {
                match cmp_result[i] {
                    Some(ordering) => M::mask_from_bool(ordering.is_le()),
                    None => M::mask_from_bool(false),
                }
            }; N]
        )
    }

    #[inline]
    fn gt(&self, other: &Packed<U, N>) -> Packed<M, N> {
        let cmp_result = self.partial_cmp(other);
        Packed::from(
            array![i => {
                match cmp_result[i] {
                    Some(ordering) => M::mask_from_bool(ordering.is_gt()),
                    None => M::mask_from_bool(false),
                }
            }; N]
        )
    }

    #[inline]
    fn ge(&self, other: &Packed<U, N>) -> Packed<M, N> {
        let cmp_result = self.partial_cmp(other);
        Packed::from(
            array![i => {
                match cmp_result[i] {
                    Some(ordering) => M::mask_from_bool(ordering.is_ge()),
                    None => M::mask_from_bool(false),
                }
            }; N]
        )
    }
}

impl <T, U, M, const N: usize> PackedPartialOrd<U, M, N> for Packed<T, N>
where
    T: Scaler + PartialOrd<U>,
    U: Scaler<MaskType = M>,
    M: Mask
{
    #[inline]
    fn partial_cmp(&self, other: &Packed<U, N>) -> [Option<Ordering>; N] {
        array![
            i => self[i].partial_cmp(&other[i]);
            N
        ]
    }
}

pub trait PackedScalerPartialOrd<U, M, const N:usize> 
where 
    U: Scaler<MaskType = M>,
    M: Mask,
{
    fn partial_cmp(&self, other: &U) -> [Option<Ordering>; N];

    #[inline]
    fn lt(&self, other: &U) -> Packed<M, N> {
        let cmp_result = self.partial_cmp(other);
        Packed::from(
            array![i => {
                match cmp_result[i] {
                    Some(ordering) => M::mask_from_bool(ordering.is_lt()),
                    None => M::mask_from_bool(false),
                }
            }; N]
        )
    }

    #[inline]
    fn le(&self, other: &U) -> Packed<M, N> {
        let cmp_result = self.partial_cmp(other);
        Packed::from(
            array![i => {
                match cmp_result[i] {
                    Some(ordering) => M::mask_from_bool(ordering.is_le()),
                    None => M::mask_from_bool(false),
                }
            }; N]
        )
    }

    #[inline]
    fn gt(&self, other: &U) -> Packed<M, N> {
        let cmp_result = self.partial_cmp(other);
        Packed::from(
            array![i => {
                match cmp_result[i] {
                    Some(ordering) => M::mask_from_bool(ordering.is_gt()),
                    None => M::mask_from_bool(false),
                }
            }; N]
        )
    }

    #[inline]
    fn ge(&self, other: &U) -> Packed<M, N> {
        let cmp_result = self.partial_cmp(other);
        Packed::from(
            array![i => {
                match cmp_result[i] {
                    Some(ordering) => M::mask_from_bool(ordering.is_ge()),
                    None => M::mask_from_bool(false),
                }
            }; N]
        )
    }
}

impl <T, U, M, const N: usize> PackedScalerPartialOrd<U, M, N> for Packed<T, N>
where
    T: Scaler + PartialOrd<U>,
    U: Scaler<MaskType = M>,
    M: Mask
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
}

impl <M, const N: usize> Packed<M, N> 
where M: Mask
{ 
    #[inline]
    pub fn broadcast_bool(b: bool) -> Self{
        Self::broadcast_scaler(M::mask_from_bool(b))
    }

    #[inline]
    pub fn all(&self) -> bool {
        self.0.iter().all(|b| {b.to_bool()})
    }

    #[inline]
    pub fn any(&self) -> bool {
        self.0.iter().any(|b| {b.to_bool()})
    }

    #[inline]
    pub fn set(&mut self, mask: &Packed<M, N>) {
        *self = *self | *mask;
    }

    #[inline]
    pub fn clear(&mut self, mask: &Packed<M, N>) {
        *self = *self & !(*mask);
    }
}

impl <const N: usize> Packed<f64, N> {

    #[inline]
    pub fn assign_masked_f64(&mut self, values: Packed<f64, N>, mask: Packed<u64, N>){

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe {
                    self.assign_masked_f64_avx2_impl(values, mask);
                    return;
                }
            }
        }

        self.assign_masked(values, mask);
    }

    #[inline]
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "avx2")]
    unsafe fn assign_masked_f64_avx2_impl(&mut self, values: Packed<f64, N>, mask: Packed<u64, N>) {
        #[cfg(target_arch = "x86")]
        use std::arch::x86::*;
        #[cfg(target_arch = "x86_64")]
        use std::arch::x86_64::*;

        for i in 0..(N/4) {
            let mask_ptr: *const u64 = mask.0.as_ptr().wrapping_add(i * 4);
            let mask_reg = _mm256_loadu_si256(std::mem::transmute(mask_ptr));

            let value_ptr: *const f64 = values.0.as_ptr().wrapping_add(i * 4);
            let value_reg = _mm256_loadu_pd(value_ptr);

            let dest_ptr: *mut f64 = self.0.as_mut_ptr().wrapping_add(i * 4);
            _mm256_maskstore_pd(dest_ptr, mask_reg, value_reg);
        }
    }
}