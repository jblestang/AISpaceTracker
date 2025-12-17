use bevy::prelude::*;
use nalgebra::Vector3;

/// Convert TEME coordinates to Bevy coordinates with detailed debugging
/// TEME: X (vernal equinox), Y (completes right-hand), Z (north pole)
/// Bevy: X (right), Y (up), Z (forward)
pub fn teme_to_bevy(pos: Vector3<f64>, name: &str, debug: bool) -> Vec3 {
    if debug {
        let distance = pos.magnitude();
        let z_ratio = pos.z / distance;
        
        println!("[COORD] {} - TEME: X={:.2}, Y={:.2}, Z={:.2} km", 
            name, pos.x, pos.y, pos.z);
        println!("[COORD] {} - Distance: {:.2} km, Z-ratio: {:.4} (1.0=north, -1.0=south, 0.0=equator)", 
            name, distance, z_ratio);
    }
    
    // Current conversion: TEME Z (north) -> Bevy Y (up)
    // This should work, but let's verify the conversion is correct
    let bevy_pos = Vec3::new(pos.x as f32, pos.z as f32, -pos.y as f32);
    
    if debug {
        println!("[COORD] {} - Bevy: X={:.2}, Y={:.2}, Z={:.2} km", 
            name, bevy_pos.x, bevy_pos.y, bevy_pos.z);
        println!("[COORD] {} - Bevy Y (up): {:.2} km (should vary for inclined orbits)", 
            name, bevy_pos.y);
    }
    
    bevy_pos
}

/// Analyze coordinate ranges in a trajectory
pub fn analyze_trajectory_coords(points: &[Vec3], name: &str) {
    if points.is_empty() {
        return;
    }
    
    let y_min = points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
    let y_max = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max);
    let x_min = points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
    let x_max = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max);
    let z_min = points.iter().map(|p| p.z).fold(f32::INFINITY, f32::min);
    let z_max = points.iter().map(|p| p.z).fold(f32::NEG_INFINITY, f32::max);
    
    println!("[TRAJ] {} - Y range (up/down): {:.2} to {:.2} km (span: {:.2} km)", 
        name, y_min, y_max, (y_max - y_min));
    println!("[TRAJ] {} - X range: {:.2} to {:.2} km", name, x_min, x_max);
    println!("[TRAJ] {} - Z range: {:.2} to {:.2} km", name, z_min, z_max);
    
    // Check if Y range is suspiciously small
    if (y_max - y_min).abs() < 100.0 {
        println!("[WARNING] {} - Y range is very small! This suggests:", name);
        println!("  1. All satellites are in equatorial orbits (unlikely), OR");
        println!("  2. Coordinate conversion is incorrect (TEME Z -> Bevy Y might be wrong)");
    }
}

