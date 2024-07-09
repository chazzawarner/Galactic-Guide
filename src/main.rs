use std::f32::consts::PI;

mod celestial_body;
use celestial_body::{CelestialBody, CelestialBodyType, CelestialBodyId, SolarSystem};

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    pbr::CascadeShadowConfigBuilder,
};

use bevy_egui::{egui::{self, pos2}, EguiContexts, EguiPlugin};

use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

use strum::IntoEnumIterator;

use bevy_polyline::prelude::*;



#[derive(Resource, Default)]
struct AppState {
    solar_system: SolarSystem,
    selected_body: CelestialBodyId,
    drawn_bodies: Vec<Entity>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin)
        .add_plugins(LookTransformPlugin)
        .add_plugins(OrbitCameraPlugin::default())
        .add_plugins(PolylinePlugin)
        .insert_resource(ClearColor(Color::BLACK))
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
    mut polyline_materials: ResMut<Assets<PolylineMaterial>>,
    mut polylines: ResMut<Assets<Polyline>>,
    asset_server: Res<AssetServer>,
) {
    // Create a solar system, selecting default body
    let mut solar_system = SolarSystem::new();
    let selected_body = CelestialBodyId::default();

    // Spawn the bodies
    let drawn_bodies = solar_system.spawn_visible(&mut commands, &asset_server, &mut meshes, &mut materials, selected_body);

    // Create an ambient light
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::ORANGE_RED,
        brightness: 100.0,
    });

    // Create "sun" light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.0, //light_consts::lux::FULL_DAYLIGHT,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 10.0,
            ..default()
        }
        .into(),
        ..default()
    });


    // Create a camera
    commands
        .spawn(Camera3dBundle::default())
        .insert(OrbitCameraBundle::new(
            OrbitCameraController::default(),
            Vec3::new(0.0, 7., 14.0),
            Vec3::new(0., 0., 0.),
            Vec3::Y,
        ));

    // Create a test orbit polyline of an orbit with radius 15.0
    let orbit_radius = 10.0;
    let vertices = (0..=360)
        .step_by(5)
        .map(|i| {
            let angle = i as f32 * PI / 180.0;
            Vec3::new(angle.cos() * orbit_radius, 0.0, angle.sin() * orbit_radius)
        })
        .collect::<Vec<_>>();

    commands.spawn(PolylineBundle {
        polyline: polylines.add(Polyline {
            vertices: vertices.clone(),
        }),
        material: polyline_materials.add(PolylineMaterial {
            width: 3.0,
            color: Color::PURPLE,
            perspective: false,
            ..default()
        }),
        ..default()
    });

    // Add app_state to resource map
    commands.insert_resource(AppState { solar_system, selected_body, drawn_bodies });
}

// Replace bodies upon update
fn replace_bodies(mut app_state: ResMut<AppState>, mut commands: Commands, asset_server: Res<AssetServer>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Despawn the drawn bodies
    for entity in app_state.drawn_bodies.iter() {
        commands.entity(*entity).despawn();
    }

    // Spawn the new bodies
    let selected_body = app_state.selected_body.clone();
    let drawn_bodies = app_state.solar_system.spawn_visible(&mut commands, &asset_server, &mut meshes, &mut materials, selected_body);
    app_state.drawn_bodies = drawn_bodies;
}

// Orbit editor modal UI
fn orbit_editor_ui(mut contexts: EguiContexts, mut app_state: ResMut<AppState>, mut commands: Commands, asset_server: Res<AssetServer>, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    egui::Window::new("Orbit Parameters").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
        ui.separator();

        // Setup test combo box, using strum to get all the celestial body ids in a list
        let alternatives = Vec::from_iter(CelestialBodyId::iter());

        // Get app state
        let mut selected_body = app_state.selected_body;

        // Find index of selected body
        let mut selected_body_index = alternatives.iter().position(|&x| x == selected_body).unwrap();

        egui::ComboBox::from_label("Select one!").show_index(
            ui,
            &mut selected_body_index,
            alternatives.len(),
            |i| format!("{:?}", alternatives[i])
        );

        // Check if the selected index has changed, and if so, update the selected_body in AppState
        if selected_body_index != alternatives.iter().position(|&x| x == app_state.selected_body).unwrap() {
            app_state.selected_body = alternatives[selected_body_index];

            // Replace the bodies
            replace_bodies(app_state, commands, asset_server, meshes, materials);
        }

        // Display the selected celestial body
        ui.label(format!("Selected body: {:?}", alternatives[selected_body_index]));

    });
}