use std::{simd::{SupportedLaneCount, LaneCount, Simd, Mask, SimdElement, MaskElement, cmp::SimdPartialOrd, cmp::SimdPartialEq, StdFloat}, ops::{RangeBounds, Bound, Neg}, mem::{size_of, MaybeUninit}, marker::PhantomData};

#[inline]
pub fn masked_select<T, M, const N: usize>(base: Simd<T, N>, other: Simd<T, N>, mask: Mask<M, N>) -> Simd<T, N>
where 
    T: SimdElement<Mask = M>,
    M: MaskElement,
    LaneCount<N>: SupportedLaneCount,
{

    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if size_of::<T>() == 4 && N * size_of::<T>() >= 32 && is_x86_feature_detected!("avx2") {
            let mut result: MaybeUninit<Simd<T,N>> = MaybeUninit::uninit();
            unsafe {
                let result_ptr: *mut f32 = std::mem::transmute(result.as_mut_ptr());
                let mask_ptr: *const f32 = std::mem::transmute(mask.to_int().as_array().as_ptr());
                let base_ptr: *const f32 = std::mem::transmute(base.as_array().as_ptr());
                let other_ptr: *const f32 = std::mem::transmute(other.as_array().as_ptr());
                for i in 0..(32 / N / size_of::<T>()) {
                    let mask_mm = _mm256_load_ps(mask_ptr.wrapping_add((32 / 4) * i));
                    let base_mm = _mm256_load_ps(base_ptr.wrapping_add((32 / 4) * i));
                    let other_mm = _mm256_load_ps(other_ptr.wrapping_add((32 / 4) * i));

                    let result_mm = _mm256_blendv_ps(base_mm, other_mm, mask_mm);

                    _mm256_store_ps(result_ptr.wrapping_add((32 / 4) * i), result_mm);
                }

                return result.assume_init()
            }
        }

        if size_of::<T>() == 8 && N * size_of::<T>() >= 32 && is_x86_feature_detected!("avx2") {
            let mut result: MaybeUninit<Simd<T,N>> = MaybeUninit::uninit();
            unsafe {
                let result_ptr: *mut f64 = std::mem::transmute(result.as_mut_ptr());
                let mask_ptr: *const f64 = std::mem::transmute(mask.to_int().as_array().as_ptr());
                let base_ptr: *const f64 = std::mem::transmute(base.as_array().as_ptr());
                let other_ptr: *const f64 = std::mem::transmute(other.as_array().as_ptr());
                for i in 0..(32 / N / size_of::<T>()) {
                    let mask_mm = _mm256_load_pd(mask_ptr.wrapping_add((32 / 8) * i));
                    let base_mm = _mm256_load_pd(base_ptr.wrapping_add((32 / 8) * i));
                    let other_mm = _mm256_load_pd(other_ptr.wrapping_add((32 / 8) * i));

                    let result_mm = _mm256_blendv_pd(base_mm, other_mm, mask_mm);
                    // let base_masked_mm = _mm256_andnot_pd(mask_mm, base_mm);
                    // let other_masked_mm = _mm256_and_pd(mask_mm, other_mm);
                    // let result_mm = _mm256_or_pd(base_masked_mm, other_masked_mm);

                    _mm256_store_pd(result_ptr.wrapping_add((32 / 8) * i), result_mm);
                }
                return result.assume_init()
            }
        }
    }

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

#[derive(Debug)]
#[derive(Clone, Copy)]
pub struct PackedOptionalReference<'a, T, const N: usize>
where
    LaneCount<N>: SupportedLaneCount,
    
{
    inner_simd: Simd<usize, N>,
    _type_marker: PhantomData<&'a T>,
}

impl <'a, T, const N: usize> PackedOptionalReference<'a, T, N> 
where
    LaneCount<N>: SupportedLaneCount
{
    #[inline]
    pub fn splat(scaler: Option<&'a T>) -> PackedOptionalReference<'a, T, N> {
        unsafe {
            PackedOptionalReference { 
                inner_simd: Simd::splat(std::mem::transmute(scaler)), 
                _type_marker: PhantomData }
        }
    }

    #[inline]
    pub fn nones() -> PackedOptionalReference<'a, T, N> {
        Self::splat(None)
    }

    #[inline]
    pub fn assign_masked(&mut self, values: &PackedOptionalReference<T, N>, mask: Mask<<usize as SimdElement>::Mask, N>) {
        masked_assign(&mut self.inner_simd, values.inner_simd, mask);
    }

    #[inline]
    pub fn at(&self, index: usize) -> Option<&'a T> {
        unsafe {
            std::mem::transmute(self.inner_simd[index])
        }
    }
}

pub fn negate_simd_float<T, const N: usize>(value: Simd<T, N>) -> Simd<T, N>
where
    LaneCount<N>: SupportedLaneCount,
    T: SimdElement,
    Simd<T, N>: StdFloat + Neg<Output = Simd<T, N>>
{
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if size_of::<T>() == 4 && N * size_of::<T>() >= 32 && is_x86_feature_detected!("avx2") {
            let mut result: MaybeUninit<Simd<T,N>> = MaybeUninit::uninit();
            unsafe {
                let result_ptr: *mut f32 = std::mem::transmute(result.as_mut_ptr());
                let input_ptr: *const f32 = std::mem::transmute(value.as_array().as_ptr());
                for i in 0..(32 / N / size_of::<T>()) {
                    let input_mm = _mm256_load_ps(input_ptr.wrapping_add((32 / 4) * i));
                    let invert_mask_mm = _mm256_broadcast_ss(std::mem::transmute(&0x8000_0000u32));

                    let result_mm = _mm256_xor_ps(input_mm, invert_mask_mm);
                    _mm256_store_ps(result_ptr.wrapping_add((32 / 4) * i), result_mm);
                }

                return result.assume_init()
            }
        }

        if size_of::<T>() == 8 && N * size_of::<T>() >= 32 && is_x86_feature_detected!("avx2") {
            let mut result: MaybeUninit<Simd<T,N>> = MaybeUninit::uninit();
            unsafe {
                let result_ptr: *mut f64 = std::mem::transmute(result.as_mut_ptr());
                let input_ptr: *const f64 = std::mem::transmute(value.as_array().as_ptr());
                for i in 0..(32 / N / size_of::<T>()) {
                    let input_mm = _mm256_load_pd(input_ptr.wrapping_add((32 / 8) * i));
                    let invert_mask_mm = _mm256_broadcast_sd(std::mem::transmute(&0x8000_0000_0000_0000u64));

                    let result_mm = _mm256_xor_pd(input_mm, invert_mask_mm);
                    _mm256_store_pd(result_ptr.wrapping_add((32 / 8) * i), result_mm);
                }

                return result.assume_init()
            }
        }
    }

    -value
}