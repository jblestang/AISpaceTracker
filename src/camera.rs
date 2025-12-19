use bevy::prelude::*;

#[derive(Component)]
pub struct CameraController {
    pub orbit_center: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            orbit_center: Vec3::ZERO,
            distance: 15000.0,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

pub fn camera_controller_system(
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera3d>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion_events: EventReader<CursorMoved>,
    mut last_cursor_pos: Local<Option<Vec2>>,
    time: Res<Time>,
) {
    for (mut transform, mut controller) in query.iter_mut() {
        // Handle mouse drag for rotation
        if mouse_button.pressed(MouseButton::Left) {
            for event in mouse_motion_events.read() {
                if let Some(last_pos) = *last_cursor_pos {
                    let delta = event.position - last_pos;
                    controller.yaw -= delta.x * 0.001;
                    controller.pitch -= delta.y * 0.001;
                    // Clamp pitch to avoid gimbal lock
                    controller.pitch = controller.pitch.clamp(
                        -std::f32::consts::PI / 2.0 + 0.1,
                        std::f32::consts::PI / 2.0 - 0.1,
                    );
                }
                *last_cursor_pos = Some(event.position);
            }
        } else {
            *last_cursor_pos = None;
        }

        // Handle arrow keys for camera rotation
        let rotation_speed = 1.0; // radians per second
        let delta_time = time.delta_secs();
        
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            controller.yaw -= rotation_speed * delta_time;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            controller.yaw += rotation_speed * delta_time;
        }
        if keyboard_input.pressed(KeyCode::ArrowUp) {
            controller.pitch += rotation_speed * delta_time;
            // Clamp pitch to avoid gimbal lock
            controller.pitch = controller.pitch.clamp(
                -std::f32::consts::PI / 2.0 + 0.1,
                std::f32::consts::PI / 2.0 - 0.1,
            );
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) {
            controller.pitch -= rotation_speed * delta_time;
            // Clamp pitch to avoid gimbal lock
            controller.pitch = controller.pitch.clamp(
                -std::f32::consts::PI / 2.0 + 0.1,
                std::f32::consts::PI / 2.0 - 0.1,
            );
        }

        // Handle zoom with W/S keys
        let zoom_speed = 500.0; // units per second
        if keyboard_input.pressed(KeyCode::KeyW) {
            controller.distance = (controller.distance - zoom_speed * delta_time).max(1000.0);
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            controller.distance = (controller.distance + zoom_speed * delta_time).min(100000.0);
        }

        // Update camera position based on yaw and pitch
        let x = controller.distance * controller.pitch.cos() * controller.yaw.sin();
        let y = controller.distance * controller.pitch.sin();
        let z = controller.distance * controller.pitch.cos() * controller.yaw.cos();
        
        transform.translation = controller.orbit_center + Vec3::new(x, y, z);
        transform.look_at(controller.orbit_center, Vec3::Y);
    }
}

