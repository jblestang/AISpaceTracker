use bevy::prelude::*;
use chrono::{DateTime, Utc, Datelike, Timelike};

/// Calculate the sun's position in 3D space based on current date/time
/// Returns the sun's direction vector (normalized) pointing from Earth to Sun
/// 
/// The sun's position is calculated based on:
/// - Solar declination (varies with date, accounts for Earth's axial tilt)
/// - Solar hour angle (varies with time of day, longitude of solar noon)
/// 
/// In Bevy's coordinate system (Y-up, right-handed):
/// - X: East/West
/// - Y: Up/Down (North/South)
/// - Z: Forward/Back
pub fn calculate_sun_direction(current_time: DateTime<Utc>) -> Vec3 {
    // Calculate day of year (1-365/366)
    let day_of_year = current_time.ordinal() as f64;
    
    // Calculate sun's declination (angle from celestial equator)
    // This accounts for Earth's axial tilt (23.44 degrees) and seasonal variation
    // Formula: declination = 23.44° * sin(360° * (284 + day_of_year) / 365)
    let axial_tilt_deg = 23.44;
    let declination_deg = axial_tilt_deg * (360.0 * (284.0 + day_of_year) / 365.0).to_radians().sin();
    let declination = declination_deg.to_radians();
    
    // Calculate solar hour angle
    // The sun is at solar noon (hour angle = 0) at a longitude that corresponds to the current UTC time
    // Solar hour angle = 15° * (hours_since_solar_noon)
    // For UTC, solar noon occurs at longitude 0° at 12:00 UTC
    let hour = current_time.hour() as f64;
    let minute = current_time.minute() as f64;
    let second = current_time.second() as f64;
    let hours_since_midnight = hour + minute / 60.0 + second / 3600.0;
    let hours_since_solar_noon = hours_since_midnight - 12.0; // Solar noon is at 12:00
    let hour_angle = (hours_since_solar_noon * 15.0).to_radians(); // 15 degrees per hour
    
    // Convert to 3D direction vector using spherical coordinates
    // The sun's position on the celestial sphere:
    // - Declination (latitude): angle from equator, positive = north, negative = south
    // - Hour angle (longitude): angle from solar noon, positive = west, negative = east
    //
    // In Bevy's coordinate system:
    // - X: East (positive) / West (negative)
    // - Y: Up (North, positive) / Down (South, negative)
    // - Z: Forward/Back
    //
    // For a point on a sphere:
    // - x = r * cos(declination) * sin(hour_angle)  [East/West]
    // - y = r * sin(declination)                      [North/South]
    // - z = r * cos(declination) * cos(hour_angle)   [Forward/Back]
    //
    // The sun direction points from Earth center toward the sun
    // At solar noon (hour_angle = 0), sun is at longitude 0° (Greenwich)
    // Positive hour angle means sun is west (later in the day)
    let x = declination.cos() * hour_angle.sin();
    let y = declination.sin();
    let z = declination.cos() * hour_angle.cos();
    
    Vec3::new(x as f32, y as f32, z as f32).normalize()
}

/// Create a mesh for the terminator line (day/night boundary)
/// The terminator is a great circle on the sphere, perpendicular to the sun direction
pub fn create_terminator_line_mesh(earth_radius: f32, sun_direction: Vec3, resolution: usize) -> Mesh {
    use bevy::render::render_resource::PrimitiveTopology;
    
    let mut mesh = Mesh::new(PrimitiveTopology::LineStrip, Default::default());
    
    let mut positions = Vec::new();
    
    // The terminator is the intersection of the sphere with a plane
    // The plane is perpendicular to the sun direction and passes through Earth's center
    // We need to find a circle on the sphere that's perpendicular to sun_direction
    
    // Find two perpendicular vectors to sun_direction to define the circle plane
    let up = Vec3::Y;
    let right = if sun_direction.dot(up).abs() > 0.9 {
        // If sun is near vertical, use a different reference
        Vec3::X
    } else {
        up
    };
    
    // Create an orthonormal basis for the circle plane
    let u = sun_direction.cross(right).normalize();
    let v = sun_direction.cross(u).normalize();
    
    // Generate points along the circle
    for i in 0..=resolution {
        let angle = (i as f32 / resolution as f32) * 2.0 * std::f32::consts::PI;
        let point_on_circle = u * angle.cos() + v * angle.sin();
        let point_on_sphere = point_on_circle * earth_radius;
        positions.push([point_on_sphere.x, point_on_sphere.y, point_on_sphere.z]);
    }
    
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh
}

#[derive(Component)]
pub struct TerminatorLine;

