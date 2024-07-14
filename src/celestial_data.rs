/// Sets radii of bodies and pull ephemeral data for their positions

use crate::celestial_body::{CelestialBodyId, SolarSystem, CelestialBodyType};

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
pub fn get_position(target_body: CelestialBodyId, central_body: CelestialBodyId, time: &Epoch, cosm: &Arc<Cosm>) -> Vec3 {
    //let epoch = Epoch::from_gregorian_utc(2024, 7, 4, 12, 0, 0, 0);
    let target = get_ephem_path(target_body);
    let frame = get_frame(central_body, cosm);
    let correction = LightTimeCalc::Abberation;

    //let frame_list = cosm.frames_get_names();
    //println!("Ephem path for target: {:?}", target);
    //println!("Frame: {:?}", frame);

    let cosm_ref = cosm.as_ref();
    let orbit = cosm_ref.celestial_state(target, *time, frame, correction);

    // Return the position of the target body
    //Vec3::new(orbit.x as f32, orbit.y as f32, orbit.z as f32) * SOLAR_SYSTEM_SCALE  
    //Vec3::new(orbit.x as f32, orbit.z as f32, orbit.y as f32) * SOLAR_SYSTEM_SCALE // Y and Z are swapped

    // Transform from J2000 to EME2000
    let pos = Vec3::new(orbit.x as f32, orbit.z as f32, orbit.y as f32) * SOLAR_SYSTEM_SCALE;
    //equatorial_to_ecliptic(pos, central_body) Need to put this somewhere else
    pos
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

// Axial tilt of celestial bodies
pub fn get_axial_tilt(body: CelestialBodyId) -> f32 {
    match body {
        CelestialBodyId::Sun => 7.25,
        CelestialBodyId::Mercury => 0.03,
        CelestialBodyId::Venus => 2.64,
        CelestialBodyId::Earth => 23.4,
        CelestialBodyId::Moon => 6.7,
        CelestialBodyId::Mars => 25.2,
        CelestialBodyId::Jupiter => 3.1,
        CelestialBodyId::Saturn => 26.7,
        CelestialBodyId::Uranus => 82.2,
        CelestialBodyId::Neptune => 28.3,
    }
}


// Transform from J200 equator to ecliptic
pub fn equatorial_to_ecliptic(pos: Vec3, central_body: CelestialBodyId) -> Vec3 {
    let x = pos.x;
    let y = pos.y;
    let z = pos.z;

    let axial_tilt = get_axial_tilt(central_body) * std::f32::consts::PI / 180.0;
    //let obliquity = 23.439281 * std::f32::consts::PI / 180.0; // Need to define axial tilt for all bodies!!

    let x_prime = x;
    let y_prime = y * axial_tilt.cos() - z * axial_tilt.sin();
    let z_prime = y * axial_tilt.sin() + z * axial_tilt.cos();

    Vec3::new(x_prime as f32, y_prime as f32, z_prime as f32)
}


// Get the trajectory of a celestial body relative to a central body over a time period with a given number of steps
// Will require refactoring to handle CelstialBody/SolarSystem struct
pub fn get_traj(target_body: CelestialBodyId, central_body: CelestialBodyId, start_time: Epoch, end_time: Epoch, steps: usize, solar_system: &SolarSystem, cosm: &Arc<Cosm>) -> Vec<Vec3> {
    let target_body_type = &solar_system.bodies.get(&target_body).unwrap().body_type;

    if target_body == CelestialBodyId::Sun {
        // Return no orbit trajectory if the target body is the sun
        return Vec::new();
    } else if target_body_type == &CelestialBodyType::Planet {
        // If not and it is a planet, find trajectory in the sun frame, transforming the coordinates to the central body frame at start_time
        let central_frame = get_frame(central_body, cosm);
        let sun_frame = get_frame(CelestialBodyId::Sun, cosm);

        // Find the position of the central body at the start time in the sun frame
        let central_position = get_position(central_body, CelestialBodyId::Sun, &start_time, cosm);

        let mut trajectory = Vec::new();
        let time_step = (end_time - start_time) / steps as f64;

        // Iterate through each step, finding the position of the target body in the sun frame and transforming to the central body frame
        for i in 0..steps {
            let time = start_time + time_step * i as f64;

            // Find the position of the target body in the sun frame
            let target_position = get_position(target_body, CelestialBodyId::Sun, &time, cosm);

            // Transform the position to the central body frame
            let target_position_central = target_position - central_position;

            // Add the position to the trajectory
            trajectory.push(target_position_central);
        }

        trajectory.push(Vec3::ZERO); // Add the final position to the trajectory

        trajectory
    } else {
        // Else, must be a moon/asteroid. So, find the trajectory in the central body frame
        let central_frame = get_frame(central_body, cosm);

        let mut trajectory = Vec::new();
        let time_step = (end_time - start_time) / steps as f64;

        // Iterate through each step, finding the position of the target body in the central body frame
        for i in 0..steps {
            let time = start_time + time_step * i as f64;

            // Find the position of the target body in the central body frame
            let target_position = get_position(target_body, central_body, &time, cosm);

            // Add the position to the trajectory
            trajectory.push(target_position);
        }

        trajectory
    }
}

// Multiply a Vec3 by a nalgebra 3x3 matrix (Bevy uses f32, so we need to convert to f64 and back again)
fn multiply_vec3_by_matrix(vec: bevy::prelude::Vec3, mat: nyx_space::linalg::Matrix<f64, nyx_space::linalg::Const<3>, nyx_space::linalg::Const<3>, nyx_space::linalg::ArrayStorage<f64, 3, 3>>) -> bevy::prelude::Vec3 {
    let x = vec.x as f64 * mat[(0, 0)] + vec.y as f64 * mat[(0, 1)] + vec.z as f64 * mat[(0, 2)];
    let y = vec.x as f64 * mat[(1, 0)] + vec.y as f64 * mat[(1, 1)] + vec.z as f64 * mat[(1, 2)];
    let z = vec.x as f64 * mat[(2, 0)] + vec.y as f64 * mat[(2, 1)] + vec.z as f64 * mat[(2, 2)];

    bevy::prelude::Vec3::new(x as f32, y as f32, z as f32)
}