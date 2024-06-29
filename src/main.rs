use std::f32::consts::PI;

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
) {
    // Setup debug material
    let debug_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..default()
    });

    // Create a cube with debug material
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::default()),
        material: debug_material.clone(),
        transform: Transform::from_xyz(0., 0., 3.)
        .with_rotation(Quat::from_rotation_x(-PI / 4.)),
        ..Default::default()
    });

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

    // Create a camera
    /*commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 7., 14.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ..default()
    });*/
    commands
        .spawn(Camera3dBundle::default())
        .insert(OrbitCameraBundle::new(
            OrbitCameraController::default(),
            Vec3::new(0.0, 7., 14.0),
            Vec3::new(0., 1., 0.),
            Vec3::Y,
        ));
}

// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

// Orbit editor modal UI
fn orbit_editor_ui(mut contexts: EguiContexts) {
    egui::Window::new("Orbit Parameters").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
    });
}