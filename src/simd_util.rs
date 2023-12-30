use std::{simd::{SupportedLaneCount, LaneCount, Simd, Mask, SimdElement, MaskElement, cmp::SimdPartialOrd, cmp::SimdPartialEq}, ops::{RangeBounds, Bound}};

#[inline]
pub fn masked_select<T, M, const N: usize>(base: Simd<T, N>, other: Simd<T, N>, mask: Mask<M, N>) -> Simd<T, N>
where 
    T: SimdElement<Mask = M>,
    M: MaskElement,
    LaneCount<N>: SupportedLaneCount,
{
    mask.select(other, base)
}

#[inline]
pub fn masked_assign<T, M, const N: usize>(base: &mut Simd<T,N>, other: Simd<T, N>, mask: Mask<M, N>)
where 
    T: SimdElement<Mask = M>,
    M: MaskElement,
    LaneCount<N>: SupportedLaneCount,
{
    *base = masked_select(*base, other, mask);
}

pub enum BoundType {
    UPPER,
    LOWER,
}

#[inline]
pub fn simd_bound_check<T, M, const N: usize>(bound_type: BoundType, values: &Simd<T,N>, bound: Bound<&T>) -> Mask<M, N>
where 
    T: SimdElement,
    M: MaskElement,
    Simd<T, N>: SimdPartialOrd + SimdPartialEq<Mask = Mask<M, N>>,
    LaneCount<N>: SupportedLaneCount
{
    match bound_type {
        BoundType::UPPER => {
            match bound {
                Bound::Included(limit) => {
                    values.simd_le(Simd::splat(*limit))
                },
                Bound::Excluded(limit) => {
                    values.simd_lt(Simd::splat(*limit))
                },
                Bound::Unbounded => {
                    Mask::splat(true)
                },
            }
        },
        BoundType::LOWER => {
            match bound {
                Bound::Included(limit) => {
                    values.simd_ge(Simd::splat(*limit))
                },
                Bound::Excluded(limit) => {
                    values.simd_gt(Simd::splat(*limit))
                },
                Bound::Unbounded => {
                    Mask::splat(true)
                },
            }
        },
    }
}

#[inline]
pub fn simd_inside<T, M, R, const N:usize> (values: &Simd<T,N>, range: &R) -> Mask<M, N>
where 
    T: SimdElement,
    M: MaskElement,
    Simd<T, N>: SimdPartialOrd + SimdPartialEq<Mask = Mask<M, N>>,
    R: RangeBounds<T>,
    LaneCount<N>: SupportedLaneCount
{
    simd_bound_check(BoundType::LOWER, values, range.start_bound()) & simd_bound_check(BoundType::UPPER, values, range.end_bound())
}