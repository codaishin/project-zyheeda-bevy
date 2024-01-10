pub mod inventory_screen;

use bevy::{hierarchy::ChildBuilder, ui::Style};

pub trait SpawnAble {
	fn spawn() -> (Style, Self);
	fn children(parent: &mut ChildBuilder);
}
