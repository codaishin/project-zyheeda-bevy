pub mod inventory_screen;
pub mod ui_overlay;

use bevy::{hierarchy::ChildBuilder, ui::Style};

pub trait SpawnAble {
	fn spawn() -> (Style, Self);
	fn children(parent: &mut ChildBuilder);
}
