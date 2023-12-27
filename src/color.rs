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
        let scale_factor = 255.999;
        let ir = (self.red.sqrt() * scale_factor) as u8;
        let ig = (self.green.sqrt() * scale_factor) as u8;
        let ib = (self.blue.sqrt() * scale_factor) as u8;

        return [ir, ig, ib];
    }

    pub fn average<'a, F>(iter: F) -> Color
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

    pub fn from_toml(table :&toml::Table) -> Color {
        let red = table["red"].as_float().unwrap_or(0.0);
        let green = table["green"].as_float().unwrap_or(0.0);
        let blue = table["blue"].as_float().unwrap_or(0.0);

        Color::new(red, green, blue)
    }

    
}

impl std::ops::Mul<Color> for Color {
    type Output = Color;
    fn mul(self, rhs: Color) -> Self::Output {
        Color{
            red: self.red * rhs.red,
            green: self.green * rhs.green,
            blue: self.blue * rhs.blue,
        }
    }
}


impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.2}, {:.2}, {:.2})", self.red, self.green, self.blue)
    }
}
