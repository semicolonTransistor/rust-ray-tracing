
pub fn to_float(value: &toml::Value) -> Option<f64>{
    match value.as_float() {
        Some(f) => Some(f),
        None => {
            match value.as_integer() {
                Some(i) => Some(i as f64),
                None => None,
            }
        },
    }
}