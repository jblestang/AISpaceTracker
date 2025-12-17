use bevy::prelude::*;

#[derive(Component)]
pub struct EarthTexture {
    pub handle: Handle<Image>,
}

#[derive(Bundle)]
pub struct EarthBundle {
    pub mesh: Mesh3d,
    pub material: MeshMaterial3d<StandardMaterial>,
    pub transform: Transform,
    pub visibility: Visibility,
    pub earth_texture: EarthTexture,
}

impl EarthBundle {
    pub fn new(
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        asset_server: &Res<AssetServer>,
    ) -> Self {
        let earth_radius = 6371.0;
        
        // Create sphere - Bevy's Sphere should have good UV mapping
        let mesh_handle = meshes.add(Sphere::new(earth_radius));
        
        // Load Earth texture
        let texture_path = "earth_texture.jpg";
        let earth_texture_handle = asset_server.load(texture_path);
        println!("Loading Earth texture from: {}", texture_path);
        
        // Create material with texture
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(earth_texture_handle.clone()),
            base_color: Color::WHITE, // White so texture shows properly
            metallic: 0.0,
            perceptual_roughness: 0.8,
            // Use unlit material for completely uniform appearance (no lighting calculations)
            unlit: true, // This makes it always fully lit regardless of lighting
            // Reduce alpha cutoff to help with texture blending at seams
            alpha_mode: AlphaMode::Opaque,
            ..default()
        });

        Self {
            mesh: Mesh3d(mesh_handle),
            material: MeshMaterial3d(material),
            transform: Transform::from_translation(Vec3::ZERO),
            visibility: Visibility::default(),
            earth_texture: EarthTexture {
                handle: earth_texture_handle,
            },
        }
    }
}

/// System to verify texture loaded and update material if needed
pub fn check_earth_texture_loaded(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    query: Query<(&EarthTexture, &MeshMaterial3d<StandardMaterial>)>,
    mut has_logged: Local<bool>,
) {
    for (earth_texture, material_3d) in query.iter() {
        match images.get_mut(&earth_texture.handle) {
            Some(image) => {
                if !*has_logged {
                    println!("✓ Earth texture loaded successfully! Size: {}x{}", 
                        image.size().x, image.size().y);
                    *has_logged = true;
                }
                
                // Note: Texture seam artifacts are inherent to equirectangular projections
                // The best solution is to use a texture that's designed to wrap seamlessly
                // or process the texture to remove the seam before using it
                
                // Ensure material is using the texture
                if let Some(material) = materials.get_mut(&material_3d.0) {
                    if material.base_color_texture.is_none() {
                        material.base_color_texture = Some(earth_texture.handle.clone());
                    }
                    material.base_color = Color::WHITE;
                }
            }
            None => {
                if !*has_logged {
                    println!("⏳ Earth texture still loading...");
                }
            }
        }
    }
}

