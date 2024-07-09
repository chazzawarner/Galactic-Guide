use std::collections::HashMap;

use strum_macros::EnumIter;

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};


// Celestial body types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CelestialBodyType {
    Star,
    Planet,
    Moon,
    Asteroid,
}
// All possible celestial body ids
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Copy, EnumIter)]
pub enum CelestialBodyId {
    Sun,
    Mercury,
    Venus,
    #[default]
    Earth,
    Moon,
    Mars,
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
}


// Celestial body struct
#[derive(Debug, Clone)]
pub struct CelestialBody {
    name: String,
    body_type: CelestialBodyType,
    pub radius: f32,
    pub position: Vec3, // Potentially uneccessary to have position here and instead store it in the render component
    parent_id: Option<CelestialBodyId>,
}

impl CelestialBody {
    pub fn new(name: &str, body_type: CelestialBodyType, radius: f32, position: Vec3, parent_id: Option<CelestialBodyId>) -> CelestialBody {
        CelestialBody {
            name: name.to_string(),
            body_type,
            radius,
            position,
            parent_id,
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
        materials: &mut ResMut<Assets<StandardMaterial>>,
        position: Vec3,
    ) -> Entity {
        let material_handle = self.create_texture(asset_server, materials);
        let mesh_handle = meshes.add(Sphere::new(self.radius).mesh().uv(32, 18));

        // Spawn the entity and get the EntityCommands
        let entity_commands = commands.spawn(PbrBundle {
            mesh: mesh_handle,
            material: material_handle,
            // Rotation is to correct for a quirk of the UV mapping making the texture upside down
            transform: Transform::from_translation(position)
                .mul_transform(Transform::from_rotation(Quat::from_rotation_x(-90.0_f32.to_radians()))),
                ..default()
        });

        // Return the Entity ID
        entity_commands.id()
    }
}

#[derive(Default)]
pub struct SolarSystem {
    bodies: HashMap<CelestialBodyId, CelestialBody>,
}

impl SolarSystem {
    pub fn new() -> SolarSystem {
        let mut bodies = HashMap::new();

        // Create all bodies (There must be a better way to do this???)
        bodies.insert(CelestialBodyId::Sun, CelestialBody::new("Sun", CelestialBodyType::Star, 546.0, Vec3::new(0.0, 0.0, 0.0), None));
        bodies.insert(CelestialBodyId::Mercury, CelestialBody::new("Mercury", CelestialBodyType::Planet, 3.0, Vec3::new(15.0, 0.0, 0.0), Some(CelestialBodyId::Sun)));
        bodies.insert(CelestialBodyId::Venus, CelestialBody::new("Venus", CelestialBodyType::Planet, 5.0, Vec3::new(20.0, 0.0, 0.0), Some(CelestialBodyId::Sun)));
        bodies.insert(CelestialBodyId::Earth, CelestialBody::new("Earth", CelestialBodyType::Planet, 5.0, Vec3::new(25.0, 0.0, 0.0), Some(CelestialBodyId::Sun)));
        bodies.insert(CelestialBodyId::Moon, CelestialBody::new("Moon", CelestialBodyType::Moon, 1.0, Vec3::new(25.0, 0.0, 1.0), Some(CelestialBodyId::Earth)));
        bodies.insert(CelestialBodyId::Mars, CelestialBody::new("Mars", CelestialBodyType::Planet, 5.0, Vec3::new(30.0, 0.0, 0.0), Some(CelestialBodyId::Sun)));
        bodies.insert(CelestialBodyId::Jupiter, CelestialBody::new("Jupiter", CelestialBodyType::Planet, 8.0, Vec3::new(35.0, 0.0, 0.0), Some(CelestialBodyId::Sun)));
        bodies.insert(CelestialBodyId::Saturn, CelestialBody::new("Saturn", CelestialBodyType::Planet, 7.5, Vec3::new(40.0, 0.0, 0.0), Some(CelestialBodyId::Sun)));
        bodies.insert(CelestialBodyId::Uranus, CelestialBody::new("Uranus", CelestialBodyType::Planet, 6.0, Vec3::new(45.0, 0.0, 0.0), Some(CelestialBodyId::Sun)));
        bodies.insert(CelestialBodyId::Neptune, CelestialBody::new("Neptune", CelestialBodyType::Planet, 6.0, Vec3::new(50.0, 0.0, 0.0), Some(CelestialBodyId::Sun)));

        SolarSystem { bodies }
    }

    // Get visible bodies from a selected body id
    pub fn get_visible_bodies(&self, body_id: CelestialBodyId) -> Vec<&CelestialBody> {
        // Create a vector to store the visible bodies, starting with the selected body
        let mut visible_bodies = Vec::new();
        if let Some(body) = self.bodies.get(&body_id) {
            visible_bodies.push(body);

            // Match the body type to determine which other bodies are visible
            match body.body_type {
                CelestialBodyType::Star => {
                    // Stars can only see themselves, so we return early
                    return visible_bodies;
                },
                CelestialBodyType::Planet => {
                    // Planets can see their moons and their parent star.
                    // Add the parent star to the list of visible bodies.
                    if let Some(parent_id) = &body.parent_id {
                        if let Some(parent_body) = self.bodies.get(&parent_id) {
                            visible_bodies.push(parent_body);
                        }
                    }

                    // Add the moons of the planet to the list of visible bodies.
                    for (_, moon) in self.bodies.iter() {
                        if let Some(parent_id) = &moon.parent_id {
                            if parent_id == &body_id {
                                visible_bodies.push(moon);
                            }
                        }
                    }
                },
                CelestialBodyType::Moon => {
                    // Attempt to find the moon's parent (planet) and add it to the list of visible bodies.
                    if let Some(parent_body) = body.parent_id.as_ref().and_then(|parent_id| self.bodies.get(parent_id)) {
                        visible_bodies.push(parent_body);

                        // Attempt to find the parent's parent (star) and add it to the list of visible bodies.
                        if let Some(grandparent_body) = parent_body.parent_id.as_ref().and_then(|grandparent_id| self.bodies.get(grandparent_id)) {
                            visible_bodies.push(grandparent_body);
                        }
                    }
                },
                _ => {}
            }
        }
        visible_bodies
    }    

    // Set the positions of the bodies based on the selected body
    // Eventually based on time and empemeris data?
    pub fn set_positions(bodies: &Vec<&CelestialBody>) -> Vec<Vec3> {
        let mut body_positions = Vec::new();
        for (i, body) in bodies.iter().enumerate() {
            body_positions.push(Vec3::new(i as f32 * 117407.0, 0.0, 0.0))
        }
        body_positions
    }



    // Spawn selected and visible bodies
    pub fn spawn_visible(&mut self,
        commands: &mut Commands, 
        asset_server: &Res<AssetServer>, 
        meshes: &mut ResMut<Assets<Mesh>>, 
        materials: &mut ResMut<Assets<StandardMaterial>>,
        selected_body: CelestialBodyId
    ) -> Vec<Entity> {
        // Get the visible bodies
        let visible_bodies = self.get_visible_bodies(selected_body);

        // Set the positions of the bodies
        let body_positions = SolarSystem::set_positions(&visible_bodies);

        // Create Entity vec
        let mut drawn_bodies = Vec::new();

        // Spawn the visible bodies
        for (i, body) in visible_bodies.iter().enumerate() {
            drawn_bodies.push(body.spawn(commands, asset_server, meshes, materials, body_positions[i]));
        }

        drawn_bodies
    }
}