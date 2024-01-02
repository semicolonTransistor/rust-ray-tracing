use std::time::Duration;


#[cfg(feature = "single_precision")]
pub type Real = f32;

#[cfg(not(feature = "single_precision"))]
pub type Real = f64;


pub fn duration_as_secs_real(duration: &Duration) -> Real {
    #[cfg(feature = "single_precision")]
    {
        return duration.as_secs_f32();
    }

    #[cfg(not(feature = "single_precision"))]
    {
        return duration.as_secs_f64();
    }
}