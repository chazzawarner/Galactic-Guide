use bevy_egui::egui::debug_text::print;
use nyx_space::{
    cosmic::{
        Cosm,
        Orbit,
        Bodies,
    },
    time::{Epoch, TimeUnits},
    Spacecraft,
    md::{StateParameter, Event},
    State,
    io::gravity::HarmonicsMem,
    dynamics::{
        orbital::{
            OrbitalDynamics,
            PointMasses,
        },
        sph_harmonics::Harmonics,
        SolarPressure,
        Drag,
        SpacecraftDynamics,
    },
    propagators::Propagator,
};

fn main() {
    // Initialise cosm that stores the ephemeris
    let cosm = Cosm::de438();

    // Set the reference frames
    let eme2k = cosm.frame("EME2000"); // Earth Mean Equator and Equinox of J2000
    let iau_earth = cosm.frame("IAU_Earth"); // International Astronomical Union Earth

    // Set epoch
    let epoch = Epoch::from_gregorian_utc(2024, 7, 4, 12, 0, 0, 0); // 4th July 2024 12:00:00 UTC

    // Define initial orbit
    let orbit = Orbit::keplerian(
        6731.0, // Semi-major axis in km (same as ISS) 
        0.01, // Eccentricity
        0.0, // inclination in degrees
        0.0, // right ascension of the ascending node in degrees
        0.0, // argument of periapsis in degrees
        0.0, // true anomaly in degrees
        epoch, // epoch
        eme2k,); // reference frame

    // Create spacecraft
    let sat = Spacecraft::new(
        orbit, // The orbit the craft is on
        1000.0, // Dry mass in kg
        0.0, // Propellant mass in kg
        1.0, // Solar radiation pressure area in m^2
        15.0, // Drag area in m^2
        1.7, // Coefficient of reflectivity
        2.2); // Coefficient of drag

    // Get the position of the spacecraft at the epoch
    let sat_x = sat.value(&StateParameter::X);
    let sat_y = sat.value(&StateParameter::Y);
    let sat_z = sat.value(&StateParameter::Z);
    println!("Spacecraft position at epoch: ({:?}, {:?}, {:?})", sat_x, sat_y, sat_z);

    // Get the velocity of the spacecraft at the epoch
    let sat_vx = sat.value(&StateParameter::VX);
    let sat_vy = sat.value(&StateParameter::VY);
    let sat_vz = sat.value(&StateParameter::VZ);
    println!("Spacecraft velocity at epoch: ({:?}, {:?}, {:?})", sat_vx, sat_vy, sat_vz);

    // Attempt to load harmonics, handling potential error
    let stor = HarmonicsMem::j2_jgm3();

    let orbital_dyn = OrbitalDynamics::new(vec![
        // Note that we are only accounting for Sun, Moon and Jupiter, in addition to the integration frame's GM
        PointMasses::new(
            &[Bodies::Sun, Bodies::Luna, Bodies::JupiterBarycenter],
            cosm.clone(),
        ),
        // Specify that these harmonics are valid only in the IAU Earth frame. We're using the
        Harmonics::from_stor(iau_earth, stor, cosm.clone()),
    ]);

    // Set up SRP and Drag second, because we need to pass them to the overarching spacecraft dynamics
    let srp = SolarPressure::default(eme2k, cosm.clone());
    let drag = Drag::std_atm1976(cosm.clone());

    // Set up the spacecraft dynamics
    let sc_dyn = SpacecraftDynamics::from_models(orbital_dyn, vec![srp, drag]);

    // Set up the propagator
    let prop = Propagator::default(sc_dyn);

    // Propagate the spacecraft until the next periapsis
    let (out, traj) = prop
        .with(sat)
        .until_event(0.5 * TimeUnits::days(1), &Event::periapsis())
        .unwrap();

    // Get the position of the spacecraft at the periapsis
    let sat_x = out.value(&StateParameter::X);
    let sat_y = out.value(&StateParameter::Y);
    let sat_z = out.value(&StateParameter::Z);
    println!("Spacecraft position at periapsis: ({:?}, {:?}, {:?})", sat_x, sat_y, sat_z);

    // Get the velocity of the spacecraft at the periapsis
    let sat_vx = out.value(&StateParameter::VX);
    let sat_vy = out.value(&StateParameter::VY);
    let sat_vz = out.value(&StateParameter::VZ);
    println!("Spacecraft velocity at periapsis: ({:?}, {:?}, {:?})", sat_vx, sat_vy, sat_vz);
}
