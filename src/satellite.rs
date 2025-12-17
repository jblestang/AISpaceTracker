use bevy::prelude::*;
use chrono::{DateTime, Utc};
use sgp4::Elements;
use nalgebra::Vector3;

#[derive(Component)]
pub struct Satellite {
    pub name: String,
    pub elements: Elements,
    pub last_update: DateTime<Utc>,
    pub use_trajectory: bool,
}

#[derive(Component)]
pub struct SatelliteLabel {
    pub name: String,
}

#[derive(Component)]
pub struct SatelliteLabelParent(pub Entity);

#[derive(Component)]
pub struct SatelliteLabelEntity(pub Entity);

impl Satellite {
    pub fn new(name: String, elements: Elements) -> Self {
        Self {
            name,
            elements,
            last_update: Utc::now(),
            use_trajectory: true,
        }
    }

    pub fn update_position(&mut self, time: DateTime<Utc>) -> Option<Vector3<f64>> {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let epoch = self.elements.datetime;
            let time_naive = time.naive_utc();
            let duration = time_naive.signed_duration_since(epoch);
            let minutes_since_epoch = duration.num_seconds() as f64 / 60.0;
            
            if minutes_since_epoch.abs() > 7.0 * 24.0 * 60.0 {
                return None;
            }
            
            let constants = sgp4::Constants::from_elements(&self.elements).ok()?;
            match constants.propagate(minutes_since_epoch) {
                Ok(state) => {
                    Some(Vector3::new(state.position[0], state.position[1], state.position[2]))
                }
                Err(_) => None,
            }
        }));
        
        match result {
            Ok(position) => position,
            Err(_) => None,
        }
    }
}

#[derive(Bundle)]
pub struct SatelliteBundle {
    pub satellite: Satellite,
    pub mesh: Mesh3d,
    pub material: MeshMaterial3d<StandardMaterial>,
    pub transform: Transform,
    pub visibility: Visibility,
}

impl SatelliteBundle {
    pub fn new(
        name: String,
        elements: Elements,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) -> Self {
        let mesh_handle = meshes.add(Sphere::new(50.0));
        
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.5, 0.0),
            emissive: LinearRgba::from(Color::srgb(0.8, 0.4, 0.0)),
            // Make satellites more visible with uniform lighting
            unlit: false,
            ..default()
        });

        let mut sat = Satellite::new(name.clone(), elements);
        let initial_position = sat.update_position(chrono::Utc::now());
        let initial_translation = if let Some(pos) = initial_position {
            Vec3::new(pos.x as f32, pos.z as f32, -pos.y as f32)
        } else {
            Vec3::ZERO
        };

        Self {
            satellite: sat,
            mesh: Mesh3d(mesh_handle),
            material: MeshMaterial3d(material),
            transform: Transform::from_translation(initial_translation),
            visibility: Visibility::default(),
        }
    }
}
