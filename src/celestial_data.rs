/// Sets radii of bodies and pull ephemeral data for their positions

use crate::celestial_body::CelestialBodyId;

use nyx_space::{
    cosmic::*,
    md::ui::Arc,
    time::*,
};

use bevy::math::Vec3;

// Set the scale of the solar system
pub const SOLAR_SYSTEM_SCALE: f32 = 0.005;

// Returning radius' of celestial bodies
pub fn get_radius(body: CelestialBodyId) -> f32 {
    let radius = match body {
        CelestialBodyId::Sun => 696340.0, // All in km
        CelestialBodyId::Mercury => 2439.7,
        CelestialBodyId::Venus => 6051.8,
        CelestialBodyId::Earth => 6371.0,
        CelestialBodyId::Moon => 1737.1,
        CelestialBodyId::Mars => 3389.5,
        CelestialBodyId::Jupiter => 69911.0,
        CelestialBodyId::Saturn => 58232.0,
        CelestialBodyId::Uranus => 25362.0,
        CelestialBodyId::Neptune => 24622.0,
    };

    radius * SOLAR_SYSTEM_SCALE // Scale the radius to the game's scale
}





// Default ephemeris data
pub fn default_ephemeris() -> Arc<Cosm> {
    Cosm::de438()
}

// Get position of a celestial body relative to a central frame
pub fn get_position(target_body: CelestialBodyId, central_body: CelestialBodyId, cosm: &Arc<Cosm>) -> Vec3 {
    let epoch = Epoch::from_gregorian_utc(2024, 7, 4, 12, 0, 0, 0);
    let target = get_ephem_path(target_body);
    let frame = get_frame(central_body, cosm);
    let correction = LightTimeCalc::Abberation;

    let frame_list = cosm.frames_get_names();
    println!("Ephem path for target: {:?}", target);
    println!("Frame: {:?}", frame);

    let cosm_ref = cosm.as_ref();
    let orbit = cosm_ref.celestial_state(target, epoch, frame, correction);

    // Return the position of the target body
    //Vec3::new(orbit.x as f32, orbit.y as f32, orbit.z as f32) * SOLAR_SYSTEM_SCALE  
    //Vec3::new(orbit.x as f32, orbit.z as f32, orbit.y as f32) * SOLAR_SYSTEM_SCALE // Y and Z are swapped

    // Transform from J2000 to EME2000
    let pos = Vec3::new(orbit.x as f32, orbit.z as f32, orbit.y as f32) * SOLAR_SYSTEM_SCALE;
    equatorial_to_ecliptic(pos)
}

// Get the path of the ephemeris data for a celestial body
pub fn get_ephem_path(body: CelestialBodyId) -> &'static [usize] {
    match body {
        CelestialBodyId::Sun => Bodies::ephem_path(&Bodies::Sun),
        CelestialBodyId::Mercury => Bodies::ephem_path(&Bodies::Mercury),
        CelestialBodyId::Venus => Bodies::ephem_path(&Bodies::Venus),
        CelestialBodyId::Earth => Bodies::ephem_path(&Bodies::Earth),
        CelestialBodyId::Moon => Bodies::ephem_path(&Bodies::Luna),
        CelestialBodyId::Mars => Bodies::ephem_path(&Bodies::MarsBarycenter),
        CelestialBodyId::Jupiter => Bodies::ephem_path(&Bodies::JupiterBarycenter),
        CelestialBodyId::Saturn => Bodies::ephem_path(&Bodies::SaturnBarycenter),
        CelestialBodyId::Uranus => Bodies::ephem_path(&Bodies::UranusBarycenter),
        CelestialBodyId::Neptune => Bodies::ephem_path(&Bodies::NeptuneBarycenter),
    }
}

// Find the reference frame for a celestial body
pub fn get_frame(body: CelestialBodyId, cosm: &Arc<Cosm>) -> Frame {
    let ephem_path = get_ephem_path(body);

    let cosm_ref = cosm.as_ref();
    cosm.frame_from_ephem_path(ephem_path)
}

// Transform from J200 equator to ecliptic
pub fn equatorial_to_ecliptic(pos: Vec3) -> Vec3 {
    let x = pos.x;
    let y = pos.y;
    let z = pos.z;

    let obliquity = 23.439281 * std::f32::consts::PI / 180.0; // Need to define axial tilt for all bodies!!

    let x_prime = x;
    let y_prime = y * obliquity.cos() - z * obliquity.sin();
    let z_prime = y * obliquity.sin() + z * obliquity.cos();

    Vec3::new(x_prime as f32, y_prime as f32, z_prime as f32)
}