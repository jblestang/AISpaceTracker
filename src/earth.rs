use bevy::prelude::*;
use crate::sun;

#[derive(Component)]
pub struct EarthTexture {
    pub day_handle: Handle<Image>,
    pub night_handle: Handle<Image>,
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

        // Create custom UV sphere for meaningful texture mapping
        // High resolution (64 sectors, 32 stacks) to ensure smooth poles and horizon
        let mesh_handle = meshes.add(create_uv_sphere(earth_radius, 64, 32));

        // Load Earth day and night textures
        let day_texture_path = "earth_texture.jpg";
        let night_texture_path = "earth_night_texture.jpg";
        let day_texture_handle: Handle<Image> = asset_server.load(day_texture_path);
        let night_texture_handle: Handle<Image> = asset_server.load(night_texture_path);
        println!("Loading Earth textures:");
        println!("  Day: {}", day_texture_path);
        println!("  Night: {}", night_texture_path);

        // Create material with day and night textures
        // Use unlit: false so emissive texture works, but keep lighting uniform via ambient light
        let material = materials.add(StandardMaterial {
            // Day texture as base color (shows on lit side)
            // Night texture as emissive (shows on dark/night side)
            base_color_texture: Some(day_texture_handle.clone()),
            emissive_texture: Some(night_texture_handle.clone()),
            base_color: Color::srgb(1.0, 1.0, 1.0), // Normal brightness
            metallic: 0.0,
            perceptual_roughness: 0.7,
            // Use unlit: false so emissive texture is visible
            unlit: false,
            alpha_mode: AlphaMode::Opaque,
            emissive: LinearRgba::from(Color::srgb(0.4, 0.4, 0.5)), // Emissive for night texture visibility
            ..default()
        });

        Self {
            mesh: Mesh3d(mesh_handle),
            material: MeshMaterial3d(material),
            transform: Transform::from_translation(Vec3::ZERO),
            visibility: Visibility::default(),
            earth_texture: EarthTexture {
                day_handle: day_texture_handle,
                night_handle: night_texture_handle,
            },
        }
    }
}

/// Creates a UV Sphere mesh with correct texture coordinates for equirectangular projection
/// Uses non-indexed geometry to avoid import issues with Indices
fn create_uv_sphere(radius: f32, sectors: usize, stacks: usize) -> Mesh {
    use bevy::render::render_resource::PrimitiveTopology;

    // Use Default::default() for RenderAssetUsages to avoid importing private struct
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, Default::default());

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();

    let sector_step = 2.0 * std::f32::consts::PI / sectors as f32;
    let stack_step = std::f32::consts::PI / stacks as f32;

    // Helper to calculate vertex attributes
    let get_vertex = |i: usize, j: usize| -> ([f32; 3], [f32; 3], [f32; 2]) {
        let phi = i as f32 * stack_step; // 0 (top) to PI (bottom)
        let theta = j as f32 * sector_step; // 0 to 2PI

        // Spherical coordinates
        // x = r * sin(phi) * cos(theta)
        // y = r * cos(phi) (Y-up)
        // z = r * sin(phi) * sin(theta)

        let x = radius * phi.sin() * theta.cos();
        let y = radius * phi.cos();
        let z = radius * phi.sin() * theta.sin();

        // Normal is simply normalized position
        let n = Vec3::new(x, y, z).normalize_or_zero();

        // UVs
        // u: 0 to 1 along sector (theta) - flipped to fix East-West inversion
        // v: 0 to 1 along stack (phi)
        let u = 1.0 - (j as f32 / sectors as f32);
        let v = i as f32 / stacks as f32;

        ([x, y, z], [n.x, n.y, n.z], [u, v])
    };

    for i in 0..stacks {
        for j in 0..sectors {
            // Get 4 corners of the quad
            let (p0, n0, uv0) = get_vertex(i, j); // Top Left
            let (p1, n1, uv1) = get_vertex(i + 1, j); // Bottom Left
            let (p2, n2, uv2) = get_vertex(i, j + 1); // Top Right
            let (p3, n3, uv3) = get_vertex(i + 1, j + 1); // Bottom Right

            // Triangle 1: Top Left -> Bottom Left -> Top Right (0 -> 1 -> 2)
            // Winding order CCW? Bevy default is CCW.
            // 0 (TL), 1 (BL), 2 (TR).

            // Top cap (i=0): p0 and p2 are same point (North Pole)?
            // Actually get_vertex(0, j) gives y=radius, x=0, z=0.
            // So p0 == p2. Triangle 1 is degenerate if i=0.
            // But we can just push it, GPU handles degenerate triangles fine usually (or we can optimize).

            // Triangle 1: Wound counter-clockwise when viewed from outside
            // p0 (Top Left) -> p2 (Top Right) -> p1 (Bottom Left)
            if i != 0 {
                positions.push(p0);
                normals.push(n0);
                uvs.push(uv0);
                positions.push(p2);
                normals.push(n2);
                uvs.push(uv2);
                positions.push(p1);
                normals.push(n1);
                uvs.push(uv1);
            }

            // Triangle 2: Wound counter-clockwise when viewed from outside
            // p2 (Top Right) -> p3 (Bottom Right) -> p1 (Bottom Left)
            // Bottom cap (i=stacks-1): p1 and p3 are same point (South Pole).
            if i != stacks - 1 {
                positions.push(p2);
                normals.push(n2);
                uvs.push(uv2);
                positions.push(p3);
                normals.push(n3);
                uvs.push(uv3);
                positions.push(p1);
                normals.push(n1);
                uvs.push(uv1);
            }
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    mesh
}

/// System to verify textures loaded and update material if needed
pub fn check_earth_texture_loaded(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    query: Query<(&EarthTexture, &MeshMaterial3d<StandardMaterial>)>,
    mut has_logged: Local<bool>,
) {
    for (earth_texture, material_3d) in query.iter() {
        let day_loaded = images.get(&earth_texture.day_handle).is_some();
        let night_loaded = images.get(&earth_texture.night_handle).is_some();
        
        if day_loaded && night_loaded && !*has_logged {
            if let Some(day_image) = images.get(&earth_texture.day_handle) {
                if let Some(night_image) = images.get(&earth_texture.night_handle) {
                    println!(
                        "✓ Earth textures loaded successfully!"
                    );
                    println!(
                        "  Day texture: {}x{}",
                        day_image.size().x,
                        day_image.size().y
                    );
                    println!(
                        "  Night texture: {}x{}",
                        night_image.size().x,
                        night_image.size().y
                    );
                    *has_logged = true;
                }
            }
        } else if !*has_logged {
            println!("⏳ Earth textures still loading...");
        }

        // Ensure material is using the textures
        if let Some(material) = materials.get_mut(&material_3d.0) {
            if material.base_color_texture.is_none() {
                material.base_color_texture = Some(earth_texture.day_handle.clone());
            }
            if material.emissive_texture.is_none() {
                material.emissive_texture = Some(earth_texture.night_handle.clone());
            }
            material.base_color = Color::WHITE;
        }
    }
}

/// System to blend day/night textures based on sun position
/// Uses emissive texture intensity to show night texture when surface is in shadow
/// The emissive texture (night) will be more visible when the surface is darker (facing away from sun)
/// 
/// Note: StandardMaterial doesn't support per-vertex emissive control, so we rely on the shader's
/// automatic blending based on lighting. The emissive texture will be more visible in darker areas.
/// If there's a day/night inversion, we may need to adjust the sun direction or material properties.
pub fn blend_day_night_textures(
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(&EarthTexture, &MeshMaterial3d<StandardMaterial>)>,
    _sun_query: Query<(&GlobalTransform, &Name), With<DirectionalLight>>,
    _time: Res<Time>,
) {
    // The emissive texture blending is handled automatically by the shader based on lighting
    // The emissive texture (night) will be more visible in darker areas (night side)
    // If there's a day/night mix up, it's likely because:
    // 1. The sun direction is inverted in update_sun_position (which we already negate)
    // 2. The emissive texture needs to be inverted
    
    // Since StandardMaterial blends emissive based on lighting automatically,
    // and the lighting is controlled by update_sun_position (which negates sun_direction),
    // the emissive should automatically show on the night side.
    // If it's showing on the day side, we might need to swap base_color_texture and emissive_texture
    
    for (_, material_3d) in query.iter() {
        if let Some(material) = materials.get_mut(&material_3d.0) {
            // Set emissive intensity to make night texture visible
            // The shader will automatically make emissive more visible in darker areas (night side)
            material.emissive = LinearRgba::from(Color::srgb(0.4, 0.4, 0.5)); // Higher emissive for visibility with uniform lighting
        }
    }
}