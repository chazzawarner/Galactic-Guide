mod celestial_body;
use celestial_body::CelestialBody;

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

use bevy_egui::{egui::{self, pos2}, EguiContexts, EguiPlugin};

use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

#[derive(Resource, Default)]
struct AppState {
    bodies: Vec<CelestialBody>,
    selected_body: usize,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(LookTransformPlugin)
        .add_plugins(OrbitCameraPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, orbit_editor_ui)
        .run();
}

// Setup 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    /* // Create an earth using the CelestialBody struct
    let earth = CelestialBody::new("Earth", 5.0, Vec3::new(0.0, 0.0, 3.0));
    earth.spawn(&mut commands, &asset_server, &mut meshes, &mut materials);*/

    // Create list of bodies
    let bodies = vec![
        CelestialBody::new("Sun", 10.0, Vec3::new(0.0, 0.0, 0.0)),
        CelestialBody::new("Mercury", 0.5, Vec3::new(15.0, 0.0, 0.0)),
        CelestialBody::new("Venus", 1.0, Vec3::new(20.0, 0.0, 0.0)),
        CelestialBody::new("Earth", 1.0, Vec3::new(25.0, 0.0, 0.0)),
        CelestialBody::new("Mars", 0.5, Vec3::new(30.0, 0.0, 0.0)),
        CelestialBody::new("Jupiter", 2.0, Vec3::new(35.0, 0.0, 0.0)),
        CelestialBody::new("Saturn", 1.5, Vec3::new(40.0, 0.0, 0.0)),
        CelestialBody::new("Uranus", 1.0, Vec3::new(45.0, 0.0, 0.0)),
        CelestialBody::new("Neptune", 1.0, Vec3::new(50.0, 0.0, 0.0)),
    ];


    // Spawn the bodies
    for body in bodies.iter() {
        body.spawn(&mut commands, &asset_server, &mut meshes, &mut materials);
    }




    // Create a light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    // Create a ground plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(10.0, 10.0)),
        material: materials.add(Color::rgb(0.5, 0.5, 0.5)),
        ..default()
    });

    commands
        .spawn(Camera3dBundle::default())
        .insert(OrbitCameraBundle::new(
            OrbitCameraController::default(),
            Vec3::new(0.0, 7., 14.0),
            Vec3::new(0., 1., 0.),
            Vec3::Y,
        ));

    // Add app_state to resource map
    commands.insert_resource(AppState { bodies, selected_body: 0 });
}


// Orbit editor modal UI
fn orbit_editor_ui(mut contexts: EguiContexts, mut app_state: ResMut<AppState>) {
    egui::Window::new("Orbit Parameters").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
        ui.separator();

        // Setup test combo box
        let alternatives = ["Sun", "Mercury", "Venus", "Earth", "Mars", "Jupiter", "Saturn", "Uranus", "Neptune"];

        egui::ComboBox::from_label("Select one!").show_index(
            ui,
            &mut app_state.selected_body,
            alternatives.len(),
            |i| alternatives[i]
        );

        // Display the selected celestial body
        ui.label(format!("Selected body: {}", alternatives[app_state.selected_body]));

    });
}