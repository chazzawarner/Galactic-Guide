use std::collections::HashMap;

use strum_macros::EnumIter;

use crate::celestial_data::{get_radius, get_position, default_ephemeris, equatorial_to_ecliptic};

use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

use nyx_space::{
    cosmic::*,
    time::*,
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
    pub body_type: CelestialBodyType,
    pub radius: f32, // Potentially uneccessary to have position here and should instead store it in the render component
    pub parent_id: Option<CelestialBodyId>,
}

impl CelestialBody {
    pub fn new(name: &str, body_type: CelestialBodyType, radius: f32, parent_id: Option<CelestialBodyId>) -> CelestialBody {
        CelestialBody {
            name: name.to_string(),
            body_type,
            radius,
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

    // Return the CelestialBodyId of the body
    pub fn get_id(&self) -> CelestialBodyId {
        match self.name.as_str() {
            "Sun" => CelestialBodyId::Sun,
            "Mercury" => CelestialBodyId::Mercury,
            "Venus" => CelestialBodyId::Venus,
            "Earth" => CelestialBodyId::Earth,
            "Moon" => CelestialBodyId::Moon,
            "Mars" => CelestialBodyId::Mars,
            "Jupiter" => CelestialBodyId::Jupiter,
            "Saturn" => CelestialBodyId::Saturn,
            "Uranus" => CelestialBodyId::Uranus,
            "Neptune" => CelestialBodyId::Neptune,
            _ => CelestialBodyId::Sun,
        }
    }
}

#[derive(Default)]
pub struct SolarSystem {
    pub bodies: HashMap<CelestialBodyId, CelestialBody>,
}

impl SolarSystem {
    pub fn new() -> SolarSystem {
        let mut bodies = HashMap::new();

        // Create all bodies (There must be a better way to do this???)
        bodies.insert(CelestialBodyId::Sun, CelestialBody::new("Sun", CelestialBodyType::Star, get_radius(CelestialBodyId::Sun), None));
        bodies.insert(CelestialBodyId::Mercury, CelestialBody::new("Mercury", CelestialBodyType::Planet, get_radius(CelestialBodyId::Mercury), Some(CelestialBodyId::Sun)));
        bodies.insert(CelestialBodyId::Venus, CelestialBody::new("Venus", CelestialBodyType::Planet, get_radius(CelestialBodyId::Venus), Some(CelestialBodyId::Sun)));
        bodies.insert(CelestialBodyId::Earth, CelestialBody::new("Earth", CelestialBodyType::Planet, get_radius(CelestialBodyId::Earth), Some(CelestialBodyId::Sun)));
        bodies.insert(CelestialBodyId::Moon, CelestialBody::new("Moon", CelestialBodyType::Moon, get_radius(CelestialBodyId::Moon), Some(CelestialBodyId::Earth)));
        bodies.insert(CelestialBodyId::Mars, CelestialBody::new("Mars", CelestialBodyType::Planet, get_radius(CelestialBodyId::Mars), Some(CelestialBodyId::Sun)));
        bodies.insert(CelestialBodyId::Jupiter, CelestialBody::new("Jupiter", CelestialBodyType::Planet, get_radius(CelestialBodyId::Jupiter), Some(CelestialBodyId::Sun)));
        bodies.insert(CelestialBodyId::Saturn, CelestialBody::new("Saturn", CelestialBodyType::Planet, get_radius(CelestialBodyId::Saturn), Some(CelestialBodyId::Sun)));
        bodies.insert(CelestialBodyId::Uranus, CelestialBody::new("Uranus", CelestialBodyType::Planet, get_radius(CelestialBodyId::Uranus), Some(CelestialBodyId::Sun)));
        bodies.insert(CelestialBodyId::Neptune, CelestialBody::new("Neptune", CelestialBodyType::Planet, get_radius(CelestialBodyId::Neptune), Some(CelestialBodyId::Sun)));

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
    pub fn set_positions(bodies: &[&CelestialBody], epoch: &Epoch) -> Vec<Vec3> {
        let mut body_positions = Vec::new();

        let target_body = bodies[0].get_id(); // Presume the first body is the target body as hardcoded?

        //body_positions.push(Vec3::new(0.0, 0.0, 0.0));

        let cosm = default_ephemeris();
        //let epoch = Epoch::from_gregorian_utc(2024, 7, 4, 12, 0, 0, 0);

        for body in bodies.iter() {
            let mut position = get_position(body.get_id(), target_body, epoch, &cosm);
            position = equatorial_to_ecliptic(position, target_body);
            body_positions.push(position);

            println!("Position of {:?}: {:?}", body.name, position)
        }

        body_positions
    }



    // Spawn selected and visible bodies
    pub fn spawn_visible(&mut self,
        commands: &mut Commands, 
        asset_server: &Res<AssetServer>, 
        meshes: &mut ResMut<Assets<Mesh>>, 
        materials: &mut ResMut<Assets<StandardMaterial>>,
        selected_body: CelestialBodyId,
        epoch: &Epoch,
    ) -> Vec<Entity> {
        // Get the visible bodies
        let visible_bodies = self.get_visible_bodies(selected_body);

        // Set the positions of the bodies
        let body_positions = SolarSystem::set_positions(&visible_bodies, epoch);

        // Create Entity vec
        let mut drawn_bodies = Vec::new();

        // Spawn the visible bodies
        for (i, body) in visible_bodies.iter().enumerate() {
            drawn_bodies.push(body.spawn(commands, asset_server, meshes, materials, body_positions[i]));
        }

        drawn_bodies
    }
}