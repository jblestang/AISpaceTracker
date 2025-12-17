use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct SatelliteFilter {
    pub text: String,
}

#[derive(Component)]
pub struct FilterTextDisplay;

#[derive(Component)]
pub struct FilterInputField;

#[derive(Resource, Default)]
pub struct InputFocus {
    pub is_focused: bool,
}

pub fn setup_ui(mut commands: Commands) {
    // Spawn UI camera with order 1 (renders on top of 3D scene)
    commands.spawn((
        Camera2d::default(),
        Camera {
            order: 1, // Higher order renders on top
            ..default()
        },
    ));
    
    // Create UI root
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        })
        .with_children(|parent| {
            // Filter text input container
            parent
                .spawn((
                    Node {
                        width: Val::Px(400.0),
                        height: Val::Px(50.0),
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                ))
                .with_children(|parent| {
                    // Label
                    parent.spawn(Text::new("Filter: "));
                    
                    // Text input display
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                padding: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                            FilterTextDisplay,
                            FilterInputField,
                            Text::new(""),
                        ));
                });
        });
}

// System to check if mouse is over input field
pub fn check_input_focus(
    mut focus: ResMut<InputFocus>,
    windows: Query<&Window>,
) {
    let window = match windows.iter().next() {
        Some(w) => w,
        None => return,
    };
    
    let cursor_pos = window.cursor_position();
    
    // Check if cursor is over input field using screen coordinates
    // Input field is at top-left: x: 10-410, y: 10-60 (from top)
    focus.is_focused = if let Some(cursor) = cursor_pos {
        let input_x_min = 10.0;
        let input_x_max = 410.0;
        let input_y_min = 10.0;
        let input_y_max = 60.0;
        
        cursor.x >= input_x_min && cursor.x <= input_x_max &&
        cursor.y >= input_y_min && cursor.y <= input_y_max
    } else {
        false
    };
}

pub fn update_filter_text(
    mut filter: ResMut<SatelliteFilter>,
    mut query: Query<&mut Text, With<FilterTextDisplay>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    focus: Res<InputFocus>,
) {
    // Only process input if mouse is over input field
    if !focus.is_focused {
        return;
    }
    
    // Handle backspace
    if keyboard_input.just_pressed(KeyCode::Backspace) {
        filter.text.pop();
    }
    
    // Handle alphanumeric keys
    let shift = keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);
    
    for key in [
        KeyCode::KeyA, KeyCode::KeyB, KeyCode::KeyC, KeyCode::KeyD, KeyCode::KeyE,
        KeyCode::KeyF, KeyCode::KeyG, KeyCode::KeyH, KeyCode::KeyI, KeyCode::KeyJ,
        KeyCode::KeyK, KeyCode::KeyL, KeyCode::KeyM, KeyCode::KeyN, KeyCode::KeyO,
        KeyCode::KeyP, KeyCode::KeyQ, KeyCode::KeyR, KeyCode::KeyS, KeyCode::KeyT,
        KeyCode::KeyU, KeyCode::KeyV, KeyCode::KeyW, KeyCode::KeyX, KeyCode::KeyY,
        KeyCode::KeyZ,
        KeyCode::Digit0, KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4,
        KeyCode::Digit5, KeyCode::Digit6, KeyCode::Digit7, KeyCode::Digit8, KeyCode::Digit9,
        KeyCode::Space, KeyCode::Minus,
    ] {
        if keyboard_input.just_pressed(key) {
            if let Some(ch) = key_to_char(key, shift) {
                filter.text.push(ch);
            }
        }
    }
    
    // Update displayed text
    for mut text in query.iter_mut() {
        *text = Text::new(&filter.text);
    }
}

fn key_to_char(key: KeyCode, shift: bool) -> Option<char> {
    match key {
        KeyCode::KeyA => Some(if shift { 'A' } else { 'a' }),
        KeyCode::KeyB => Some(if shift { 'B' } else { 'b' }),
        KeyCode::KeyC => Some(if shift { 'C' } else { 'c' }),
        KeyCode::KeyD => Some(if shift { 'D' } else { 'd' }),
        KeyCode::KeyE => Some(if shift { 'E' } else { 'e' }),
        KeyCode::KeyF => Some(if shift { 'F' } else { 'f' }),
        KeyCode::KeyG => Some(if shift { 'G' } else { 'g' }),
        KeyCode::KeyH => Some(if shift { 'H' } else { 'h' }),
        KeyCode::KeyI => Some(if shift { 'I' } else { 'i' }),
        KeyCode::KeyJ => Some(if shift { 'J' } else { 'j' }),
        KeyCode::KeyK => Some(if shift { 'K' } else { 'k' }),
        KeyCode::KeyL => Some(if shift { 'L' } else { 'l' }),
        KeyCode::KeyM => Some(if shift { 'M' } else { 'm' }),
        KeyCode::KeyN => Some(if shift { 'N' } else { 'n' }),
        KeyCode::KeyO => Some(if shift { 'O' } else { 'o' }),
        KeyCode::KeyP => Some(if shift { 'P' } else { 'p' }),
        KeyCode::KeyQ => Some(if shift { 'Q' } else { 'q' }),
        KeyCode::KeyR => Some(if shift { 'R' } else { 'r' }),
        KeyCode::KeyS => Some(if shift { 'S' } else { 's' }),
        KeyCode::KeyT => Some(if shift { 'T' } else { 't' }),
        KeyCode::KeyU => Some(if shift { 'U' } else { 'u' }),
        KeyCode::KeyV => Some(if shift { 'V' } else { 'v' }),
        KeyCode::KeyW => Some(if shift { 'W' } else { 'w' }),
        KeyCode::KeyX => Some(if shift { 'X' } else { 'x' }),
        KeyCode::KeyY => Some(if shift { 'Y' } else { 'y' }),
        KeyCode::KeyZ => Some(if shift { 'Z' } else { 'z' }),
        KeyCode::Digit0 => Some('0'),
        KeyCode::Digit1 => Some('1'),
        KeyCode::Digit2 => Some('2'),
        KeyCode::Digit3 => Some('3'),
        KeyCode::Digit4 => Some('4'),
        KeyCode::Digit5 => Some('5'),
        KeyCode::Digit6 => Some('6'),
        KeyCode::Digit7 => Some('7'),
        KeyCode::Digit8 => Some('8'),
        KeyCode::Digit9 => Some('9'),
        KeyCode::Space => Some(' '),
        KeyCode::Minus => Some(if shift { '_' } else { '-' }),
        _ => None,
    }
}

pub fn filter_satellites(
    filter: Res<SatelliteFilter>,
    mut satellite_query: Query<(&mut Visibility, &crate::satellite::Satellite, Option<&crate::satellite::SatelliteLabelEntity>)>,
    mut label_query: Query<&mut Visibility, (With<crate::satellite::SatelliteLabel>, Without<crate::satellite::Satellite>)>,
) {
    // Only update if filter changed
    if !filter.is_changed() {
        return;
    }
    
    let filter_lower = filter.text.to_lowercase();
    
    for (mut visibility, satellite, label_entity) in satellite_query.iter_mut() {
        let should_show = if filter.text.is_empty() {
            // Show all if filter is empty
            true
        } else {
            // Partial match (case-insensitive)
            satellite.name.to_lowercase().contains(&filter_lower)
        };
        
        // Update satellite visibility
        *visibility = if should_show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        
        // Update label visibility to match satellite
        if let Some(label_entity) = label_entity {
            if let Ok(mut label_visibility) = label_query.get_mut(label_entity.0) {
                *label_visibility = *visibility;
            }
        }
    }
}
