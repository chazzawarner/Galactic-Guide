mod celestial_body;
use celestial_body::CelestialBody;

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

use bevy_egui::{egui, EguiContexts, EguiPlugin};

use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

struct AppState {
    bodies: Vec<CelestialBody>,
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
    // Create an earth using the CelestialBody struct
    let earth = CelestialBody::new("Earth", 5.0, Vec3::new(0.0, 0.0, 3.0));
    earth.spawn(&mut commands, &asset_server, &mut meshes, &mut materials);
    

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
}


// Orbit editor modal UI
fn orbit_editor_ui(mut contexts: EguiContexts) {
    egui::Window::new("Orbit Parameters").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
    });
}