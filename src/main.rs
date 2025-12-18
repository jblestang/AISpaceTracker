use bevy::prelude::*;
use bevy::pbr::wireframe::WireframePlugin;
use chrono::{DateTime, Utc};
use sgp4::Elements;

mod satellite;
mod earth;
mod camera;
mod tle_loader;
mod coordinate_debug;
mod ui;

use satellite::{Satellite, SatelliteBundle};
use earth::{EarthBundle, EarthTexture};
use camera::CameraController;
use tle_loader::TleLoader;
use coordinate_debug::teme_to_bevy;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "AI Space Tracker - Live Satellite Tracker".into(),
                resolution: (1920, 1080).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(WireframePlugin::default())
        .init_resource::<ui::SatelliteFilter>()
        .init_resource::<ui::InputFocus>()
        .add_systems(Startup, (setup_scene, load_satellites, ui::setup_ui))
        .add_systems(Update, (
            update_satellite_positions,
            update_satellite_labels,
            earth::check_earth_texture_loaded,
            camera::camera_controller_system,
            ui::check_input_focus,
            ui::update_filter_text,
            ui::filter_satellites,
        ))
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Spawn Earth
    commands.spawn(EarthBundle::new(&mut meshes, &mut materials, &asset_server));

    // Use high ambient light for uniform lighting
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 5.0, // Very high brightness for uniform lighting
        affects_lightmapped_meshes: false,
    });
    
    // Add a very soft directional light from all directions (no shadows, no dark areas)
    // This helps with visibility while keeping lighting uniform
    commands.spawn((
        DirectionalLight {
            illuminance: 3000.0, // Moderate brightness
            shadows_enabled: false, // No shadows
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0)),
    ));

    // Spawn camera with order 0 (3D scene)
    commands.spawn((
        Camera3d::default(),
        Camera::default(),
        Transform::from_xyz(0.0, 0.0, 15000.0)
        .looking_at(Vec3::ZERO, Vec3::Y),
        CameraController {
            orbit_center: Vec3::ZERO,
            distance: 15000.0,
            ..default()
        },
    ));
}

fn load_satellites(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Load TLE data from Celestrak (open source satellite data)
    let tle_loader = TleLoader::new();
    
    // Load popular satellites (ISS, Starlink, etc.)
    if let Ok(satellites) = tle_loader.load_active_satellites() {
        for (name, tle_data) in satellites.iter().take(10000) {
            // Limit to 10000 satellites
            if let Ok(elements) = tle_data.to_elements() {
                let bundle = SatelliteBundle::new(
                    name.clone(),
                    elements,
                    &mut meshes,
                    &mut materials,
                );
                let satellite_entity = commands.spawn(bundle).id();
                
                // Spawn text label - we'll position it manually each frame since Text2d is screen-space
                let label_entity = commands.spawn((
                    Text2d::new(name.clone()),
                    Transform::default(),
                    satellite::SatelliteLabel {
                        name: name.clone(),
                    },
                    satellite::SatelliteLabelParent(satellite_entity),
                    Visibility::Visible,
                )).id();
                
                // Store label entity reference on satellite for easy lookup
                commands.entity(satellite_entity).insert(satellite::SatelliteLabelEntity(label_entity));
            }
        }
    }
}

fn update_satellite_positions(
    mut query: Query<(&mut Transform, &mut Satellite)>,
    time: Res<Time>,
) {
    // Time acceleration factor - satellites move this many times faster than real-time
    // Set to 1.0 for real-time (no acceleration)
    const TIME_ACCELERATION: f64 = 1.0;
    
    // Calculate accelerated time based on app start time
    // Use fractional seconds for smooth animation
    static mut START_TIME: Option<DateTime<Utc>> = None;
    let start_time = unsafe {
        if START_TIME.is_none() {
            START_TIME = Some(Utc::now());
        }
        START_TIME.unwrap()
    };
    
    // Use elapsed time with fractional seconds for smooth movement
    let elapsed_seconds = time.elapsed().as_secs_f64();
    let accelerated_seconds = elapsed_seconds * TIME_ACCELERATION;
    
    // Create duration with nanosecond precision for smooth animation
    // This ensures we don't lose precision when converting to DateTime
    let total_nanos = (accelerated_seconds * 1_000_000_000.0) as i64;
    let accelerated_time = start_time + chrono::Duration::nanoseconds(total_nanos);
    
    for (mut transform, mut satellite) in query.iter_mut() {
        if let Some(position) = satellite.update_position(accelerated_time) {
            // Convert TEME to Bevy using debug function
            // Enable debug for first few satellites
            static mut DEBUG_COUNT: usize = 0;
            let debug = unsafe {
                if DEBUG_COUNT < 3 {
                    DEBUG_COUNT += 1;
                    true
                } else {
                    false
                }
            };
            transform.translation = teme_to_bevy(position, &satellite.name, debug);
        }
    }
}

// System to update label positions to follow satellites
// Text2d renders in screen space, so we need to project 3D positions to screen coordinates
fn update_satellite_labels(
    mut label_query: Query<(&mut Transform, &mut Visibility, &satellite::SatelliteLabelParent), With<satellite::SatelliteLabel>>,
    satellite_query: Query<(&GlobalTransform, &Visibility), (With<satellite::Satellite>, Without<satellite::SatelliteLabel>)>,
    camera_query: Query<&GlobalTransform, (With<Camera3d>, Without<satellite::SatelliteLabel>)>,
    windows: Query<&Window>,
    camera: Query<&Camera, (With<Camera3d>, Without<satellite::SatelliteLabel>)>,
) {
    // Get camera and window for projection
    let camera_global = match camera_query.iter().next() {
        Some(c) => c,
        None => return,
    };
    
    let camera_comp = match camera.iter().next() {
        Some(c) => c,
        None => return,
    };
    
    let window = match windows.iter().next() {
        Some(w) => w,
        None => return,
    };
    
            let camera_pos = camera_global.translation();
            let earth_center = Vec3::ZERO;
            let earth_radius = 6371.0;
            
            for (mut label_transform, mut visibility, parent) in label_query.iter_mut() {
                // Get satellite's world position and visibility
                if let Ok((sat_global, sat_visibility)) = satellite_query.get(parent.0) {
                    // If satellite is hidden (filtered out), hide label too
                    if *sat_visibility == Visibility::Hidden {
                        *visibility = Visibility::Hidden;
                        continue;
                    }
                    
                    let sat_pos = sat_global.translation();
            
            // Check if satellite is behind Earth from camera's perspective
            let camera_to_sat = sat_pos - camera_pos;
            let camera_to_earth = earth_center - camera_pos;
            let sat_to_earth = earth_center - sat_pos;
            
            // Check if satellite is inside Earth (shouldn't happen, but hide label)
            if sat_to_earth.length() < earth_radius {
                *visibility = Visibility::Hidden;
                continue;
            }
            
            // Check if satellite is behind Earth using ray-sphere intersection
            let camera_to_sat_dir = camera_to_sat.normalize();
            let camera_to_earth_vec = camera_to_earth;
            
            // Find closest point on camera->satellite line to Earth center
            let t = camera_to_earth_vec.dot(camera_to_sat_dir);
            let closest_point = camera_pos + camera_to_sat_dir * t;
            let distance_to_earth_center = (closest_point - earth_center).length();
            
            // If the line from camera to satellite passes through Earth
            if distance_to_earth_center < earth_radius {
                // Calculate intersection points using ray-sphere intersection
                let half_chord = (earth_radius * earth_radius - distance_to_earth_center * distance_to_earth_center).sqrt();
                let t_entry = t - half_chord; // Where ray enters Earth
                let t_exit = t + half_chord;  // Where ray exits Earth
                
                let camera_sat_dist = camera_to_sat.length();
                
                // If satellite is beyond the exit point (behind Earth), hide label
                if camera_sat_dist > t_exit && t_exit > 0.0 {
                    *visibility = Visibility::Hidden;
                    continue;
                }
            }
            
            // Additional check: if satellite is on the opposite side of Earth from camera
            let earth_to_sat = sat_pos - earth_center;
            let camera_to_earth_dir = camera_to_earth.normalize();
            let earth_to_sat_dir = earth_to_sat.normalize();
            
            // Check if satellite is behind Earth using dot product
            // Negative dot product means satellite is on opposite side
            let dot = camera_to_earth_dir.dot(earth_to_sat_dir);
            if dot < -0.2 {
                let camera_earth_dist = camera_to_earth.length();
                let sat_earth_dist = sat_to_earth.length();
                
                // Only apply this check if both are reasonably far from Earth
                if camera_earth_dist > earth_radius * 1.2 && sat_earth_dist > earth_radius * 1.2 {
                    *visibility = Visibility::Hidden;
                    continue;
                }
            }
            
            // Position label below satellite in world space
            let world_pos = sat_pos + Vec3::new(0.0, -150.0, 0.0);
            
            // Project 3D world position to 2D screen coordinates
            if let Some(ndc) = camera_comp.world_to_ndc(camera_global, world_pos) {
                // Check if point is behind camera or outside view frustum
                if ndc.z > 1.0 || ndc.z < -1.0 {
                    // Point is behind camera or too far, hide label
                    *visibility = Visibility::Hidden;
                    continue;
                }
                
                // Convert NDC to screen coordinates
                // NDC ranges from -1 to 1, screen coordinates from 0 to width/height
                let screen_x = (ndc.x + 1.0) * 0.5 * window.width();
                let screen_y = (1.0 - ndc.y) * 0.5 * window.height(); // Flip Y axis (screen Y is top-down)
                
                // Text2d uses 2D camera space coordinates
                // 2D camera is centered at (0, 0) with window dimensions
                // Convert screen coordinates to 2D camera world coordinates
                let camera2d_x = screen_x - window.width() * 0.5;
                let camera2d_y = window.height() * 0.5 - screen_y; // Flip Y for 2D camera (Y up)
                
                // Set label position in 2D camera space
                label_transform.translation = Vec3::new(camera2d_x, camera2d_y, 0.0);
                label_transform.scale = Vec3::splat(0.5); // Smaller scale for better readability
                *visibility = Visibility::Visible;
            } else {
                // Point is not visible, hide label
                *visibility = Visibility::Hidden;
            }
        }
    }
}

