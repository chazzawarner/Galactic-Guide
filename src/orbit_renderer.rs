/// Using the polyline plugin to render the orbits of celestial bodies and spacecraft

use crate::celestial_data::{get_traj, equatorial_to_ecliptic, get_orbital_period};
use crate::celestial_body::{CelestialBodyId, SolarSystem};

use bevy_polyline::prelude::*;

use bevy::prelude::*;

use nyx_space::{
    cosmic::Cosm,
    time::Epoch,
    md::ui::Arc,
};

const TRAJ_POINTS: usize = 1000;
const TRAJ_COLOURS: [Color; 10] = [
    Color::RED,
    Color::GREEN,
    Color::BLUE,
    Color::YELLOW,
    Color::CYAN,
    Color::PURPLE,
    Color::WHITE,
    Color::GRAY,
    Color::BLACK,
    Color::PINK,
];

// Plot the trajectories of the visible celestial bodies
pub fn render_trajs(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<PolylineMaterial>>,
    polylines: &mut ResMut<Assets<Polyline>>,
    solar_system: &SolarSystem,
    selected_body: CelestialBodyId,
    cosm: Arc<Cosm>,
    epoch: Epoch,
) {
    // Get the list of selected bodies
    let bodies = solar_system.get_visible_bodies(selected_body);

    // Plot the trajectory for each body
    for (index, body) in bodies.iter().enumerate() {
        let end_epoch = epoch + get_orbital_period(body.get_id());
        let traj = get_traj(body.get_id(), selected_body, epoch, end_epoch, TRAJ_POINTS, solar_system, &cosm);

        let mut points = Vec::new();
        for point in traj.iter() {
            let mut scaled_point = Vec3::new(
                point[0] as f32, 
                point[1] as f32, 
                point[2] as f32);
            scaled_point = equatorial_to_ecliptic(scaled_point, selected_body);
            points.push(scaled_point);
        }

        // Use modulo to loop through colors if there are more bodies than colors
        let color = TRAJ_COLOURS[index % TRAJ_COLOURS.len()];

        commands.spawn(PolylineBundle {
            polyline: polylines.add(Polyline {
                vertices: points.clone(),
            }),
            material: materials.add(PolylineMaterial {
                width: 3.0,
                color, // Use the selected color
                perspective: false,
                ..default()
            }),
            ..default()
        });
    }

}

