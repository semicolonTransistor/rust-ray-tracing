use crate::real::Real;


pub fn to_float(value: &toml::Value) -> Option<Real>{
    match value.as_float() {
        Some(f) => Some(f as Real),
        None => {
            match value.as_integer() {
                Some(i) => Some(i as Real),
                None => None,
            }
        },
    }
}