pub mod inventory_screen;
pub mod ui_overlay;

use bevy::ui::Style;

pub trait Spawn {
	fn spawn() -> (Style, Self);
}
