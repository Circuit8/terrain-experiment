use bevy::prelude::*;

/// Grabs the cursor when game first starts
pub fn initial_grab(mut windows: ResMut<Windows>) {
    toggle_grab(windows.get_primary_mut().unwrap());
}

pub fn grab(keys: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    if keys.just_pressed(KeyCode::Escape) {
        toggle_grab(window);
    }
}

/// Grabs/ungrabs mouse cursor
fn toggle_grab(window: &mut Window) {
    window.set_cursor_lock_mode(!window.cursor_locked());
    window.set_cursor_visibility(!window.cursor_visible());
}
