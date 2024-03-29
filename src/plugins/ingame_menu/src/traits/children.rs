pub mod inventory_screen;
pub mod ui_overlay;

use bevy::hierarchy::ChildBuilder;

pub trait Children {
	fn children(parent: &mut ChildBuilder);
}
