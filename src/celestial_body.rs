// Creating structure for celestial bodies such as the sun, moon, and planets
pub struct CelestialBody {
    name: String,
    pub radius: f32,
}

impl CelestialBody {
    pub fn new(name: &str, radius: f32) -> CelestialBody {
        CelestialBody {
            name: name.to_string(),
            radius,
        }
    }
}