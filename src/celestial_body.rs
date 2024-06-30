/// Creating structure for celestial bodies such as the sun, moon, and planets

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

pub struct CelestialBody {
    name: String,
    pub radius: f32,
    pub position: Vec3,
}

impl CelestialBody {
    pub fn new(name: &str, radius: f32, position: Vec3) -> CelestialBody {
        CelestialBody {
            name: name.to_string(),
            radius,
            position,
        }
    }

    // Method to create a texture for the celestial body
    fn create_texture(
        &self, 
        asset_server: &Res<AssetServer>, 
        materials: &mut ResMut<Assets<StandardMaterial>>
    ) -> Handle<StandardMaterial> {
        let image_path = format!("textures/{}.png", self.name.to_lowercase()).to_string();
        let texture_handle: Handle<Image> = asset_server.load(image_path);
        materials.add(StandardMaterial {
            base_color_texture: Some(texture_handle),
            ..default()
        })
    }

    // Method to spawn the celestial body in the world
    pub fn spawn(&self,
        commands: &mut Commands, 
        asset_server: &Res<AssetServer>, 
        meshes: &mut ResMut<Assets<Mesh>>, 
        materials: &mut ResMut<Assets<StandardMaterial>>
    ) {
        let material_handle = self.create_texture(asset_server, materials);
        let mesh_handle = meshes.add(Sphere::new(self.radius).mesh().uv(32, 18));

        commands.spawn(PbrBundle {
            mesh: mesh_handle,
            material: material_handle,
            // Rotation is to correct for a quirk of the UV mapping making the texture upside down
            transform: Transform::from_translation(self.position)
                .mul_transform(Transform::from_rotation(Quat::from_rotation_x(-90.0_f32.to_radians()))),
                ..default()
        });
    }
}