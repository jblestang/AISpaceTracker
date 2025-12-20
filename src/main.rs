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
mod sun;

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
            update_sun_position,
            update_terminator_line,
            earth::check_earth_texture_loaded,
            earth::blend_day_night_textures, // Blend day/night textures based on sun position
            camera::camera_controller_system,
            ui::check_input_focus,
            ui::update_filter_text,
            ui::filter_satellites,
            toggle_fullscreen, // Toggle fullscreen mode
        ))
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
) {
    // Spawn Earth
    commands.spawn(EarthBundle::new(&mut meshes, &mut materials, &asset_server));

    // Uniform ambient light (no day/night variation)
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.6, // Reduced brightness for less bright day
        affects_lightmapped_meshes: false,
    });
    
    // Spawn the sun as a directional light
    // The sun will be positioned and rotated based on current time using real astronomical calculations
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.85), // Warm sunlight color
            illuminance: 20000.0, // Reduced for less bright day
            shadows_enabled: false, // Disable shadows for performance
            ..default()
        },
        Transform::default(), // Will be updated by update_sun_position system
        Name::new("Sun"),
    ));
    
    // Add a secondary softer light for twilight/dawn transition gradient
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.9, 0.85, 0.7), // Softer warm light for transition
            illuminance: 5000.0, // Reduced for smoother transition
            shadows_enabled: false,
            ..default()
        },
        Transform::default(), // Will be updated by update_sun_position system
        Name::new("TwilightLight"),
    ));
    
    // Spawn terminator line (day/night boundary) as a red line
    let earth_radius = 6371.0;
    let initial_sun_dir = sun::calculate_sun_direction(get_current_time(&time));
    // Terminator is perpendicular to sun direction
    let terminator_mesh = sun::create_terminator_line_mesh(earth_radius, initial_sun_dir, 128);
    let terminator_mesh_handle = meshes.add(terminator_mesh);
    
    let terminator_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0), // Red color
        unlit: true, // Always visible regardless of lighting
        ..default()
    });
    
    commands.spawn((
        Mesh3d(terminator_mesh_handle),
        MeshMaterial3d(terminator_material),
        Transform::from_translation(Vec3::ZERO),
        sun::TerminatorLine,
        Name::new("TerminatorLine"),
    ));

    // Spawn camera with order 0 (3D scene)
    // Orient camera to focus on Europe
    // Europe is approximately at: Longitude 10°E, Latitude 50°N
    // The camera controller uses: x = distance * cos(pitch) * sin(yaw), y = distance * sin(pitch), z = distance * cos(pitch) * cos(yaw)
    // We need to set yaw and pitch to point toward Europe
    
    let europe_lon_deg: f32 = 10.0; // 10°E
    let europe_lat_deg: f32 = 50.0; // 50°N
    
    // Convert to radians
    let europe_lon_rad = europe_lon_deg.to_radians();
    let europe_lat_rad = europe_lat_deg.to_radians();
    
    // Calculate yaw and pitch for camera controller
    // Yaw: azimuth angle (0 = looking along +Z, positive rotates toward +X)
    // For Europe at 10°E, we need to rotate the camera
    // Since the camera orbits around origin, yaw controls longitude view
    // Pitch controls latitude view (0 = equator, positive = north)
    
    // The camera controller's coordinate system:
    // - yaw=0: camera at (0, 0, distance) looking at origin
    // - yaw rotates around Y axis
    // - For 10°E longitude, we need to rotate camera to face that direction
    // - Since texture might be flipped, try both positive and negative
    
    // Try: yaw = -longitude (negative because camera controller might use opposite convention)
    // Or: yaw = longitude + PI (180° offset if showing opposite side)
    // Since user reports Australia/Asia (120-150°E) when expecting Europe (10°E),
    // that's about 110-140° off, or roughly 180° - 70° = 110°
    // Let's try: yaw = -longitude - PI/2 or adjust based on actual offset
    
    let camera_distance = 15000.0;
    
    // Calculate yaw: for Europe at 10°E, adjust for coordinate system
    // The UV sphere texture is flipped East-West (U = 1.0 - u)
    // So we need to account for this in the camera positioning
    // If showing Australia/Asia (~130°E) when expecting Europe (10°E), 
    // that's about 120° off, suggesting we need to adjust by ~120° or use opposite side
    // Try: add 180° offset to get to opposite side, or adjust based on texture flip
    let yaw = -europe_lon_rad + std::f32::consts::PI; // Add 180° to account for texture flip
    
    // Pitch: slight angle to view Europe from above
    let pitch = europe_lat_rad * 0.3; // 30% of latitude for slight angle
    
    // Calculate camera position using camera controller formula
    let x = camera_distance * pitch.cos() * yaw.sin();
    let y = camera_distance * pitch.sin();
    let z = camera_distance * pitch.cos() * yaw.cos();
    let camera_position = Vec3::new(x, y, z);
    
    commands.spawn((
        Camera3d::default(),
        Camera::default(),
        Transform::from_translation(camera_position)
            .looking_at(Vec3::ZERO, Vec3::Y),
        CameraController {
            orbit_center: Vec3::ZERO,
            distance: camera_distance,
            yaw,
            pitch,
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

// Helper function to get current simulation time
fn get_current_time(time: &Time) -> DateTime<Utc> {
    const TIME_ACCELERATION: f64 = 1.0;
    
    static mut START_TIME: Option<DateTime<Utc>> = None;
    let start_time = unsafe {
        if START_TIME.is_none() {
            START_TIME = Some(Utc::now());
        }
        START_TIME.unwrap()
    };
    
    let elapsed_seconds = time.elapsed().as_secs_f64();
    let accelerated_seconds = elapsed_seconds * TIME_ACCELERATION;
    let total_nanos = (accelerated_seconds * 1_000_000_000.0) as i64;
    start_time + chrono::Duration::nanoseconds(total_nanos)
}

fn update_satellite_positions(
    mut query: Query<(&mut Transform, &mut Satellite)>,
    time: Res<Time>,
) {
    let current_time = get_current_time(&time);
    
    for (mut transform, mut satellite) in query.iter_mut() {
        if let Some(position) = satellite.update_position(current_time) {
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

/// Update sun position using real astronomical calculations
/// Accounts for Earth's axial tilt and seasonal variation
fn update_sun_position(
    mut light_query: Query<(&mut Transform, &Name), With<DirectionalLight>>,
    time: Res<Time>,
) {
    let current_time = get_current_time(&time);
    
    // Calculate real sun direction based on date/time
    // This returns a vector pointing from Earth center toward the sun
    let sun_direction = sun::calculate_sun_direction(current_time);
    
    // For a directional light in Bevy, the light direction is the direction the light is pointing
    // We want the light to point toward Earth, so the light direction should be -sun_direction
    // (from sun toward Earth, which is opposite of sun_direction which is from Earth toward sun)
    //
    // However, if day/night are inverted, we need to negate the sun direction
    // Position the light far from Earth in the direction opposite to sun_direction
    // The light's transform.forward() will point toward Earth
    let sun_distance = 50000.0; // Far enough to be effectively parallel
    // Negate sun_direction to fix day/night inversion
    let sun_position = sun_direction * sun_distance; // Position light in sun direction (inverted)
    
    // Position twilight light slightly ahead of sun for gradient effect
    // Rotate sun direction slightly for twilight
    let twilight_rotation = Quat::from_axis_angle(Vec3::Y, 0.15); // ~8.6 degrees
    let twilight_direction = twilight_rotation * sun_direction;
    let twilight_position = twilight_direction * sun_distance; // Inverted to match sun position
    
    for (mut transform, name) in light_query.iter_mut() {
        if name.as_str() == "Sun" {
            // Position the sun and make it look at Earth (center)
            transform.translation = sun_position;
            transform.look_at(Vec3::ZERO, Vec3::Y);
        } else if name.as_str() == "TwilightLight" {
            // Position twilight light for smooth transition
            transform.translation = twilight_position;
            transform.look_at(Vec3::ZERO, Vec3::Y);
        }
    }
}

/// Update terminator line (day/night boundary) based on current sun position
fn update_terminator_line(
    mut terminator_query: Query<&mut Mesh3d, (With<sun::TerminatorLine>, Without<DirectionalLight>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
) {
    let current_time = get_current_time(&time);
    // Calculate sun direction (from Earth toward sun)
    let sun_direction = sun::calculate_sun_direction(current_time);
    
    let earth_radius = 6371.0;
    
    // Recreate terminator line mesh with updated sun direction
    // The terminator is perpendicular to the sun direction
    let terminator_mesh = sun::create_terminator_line_mesh(earth_radius, sun_direction, 128);
    let new_mesh_handle = meshes.add(terminator_mesh);
    
    for mut mesh_3d in terminator_query.iter_mut() {
        // Replace the mesh handle with a new one
        *mesh_3d = Mesh3d(new_mesh_handle.clone());
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

/// Toggle fullscreen mode with F11 or Alt+Enter
fn toggle_fullscreen(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window>,
) {
    // Check for F11 or Alt+Enter
    let toggle_f11 = keyboard_input.just_pressed(KeyCode::F11);
    let toggle_alt_enter = (keyboard_input.pressed(KeyCode::AltLeft) || keyboard_input.pressed(KeyCode::AltRight))
        && keyboard_input.just_pressed(KeyCode::Enter);
    
    if toggle_f11 || toggle_alt_enter {
        for mut window in windows.iter_mut() {
            use bevy::window::{WindowMode, MonitorSelection, VideoModeSelection};
            window.mode = match window.mode {
                WindowMode::Windowed => WindowMode::Fullscreen(MonitorSelection::Current, VideoModeSelection::Current),
                _ => WindowMode::Windowed,
            };
        }
    }
}

