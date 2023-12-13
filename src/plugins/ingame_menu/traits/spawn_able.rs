pub mod inventory_screen;

use super::colors::BaseColors;
use bevy::{hierarchy::ChildBuilder, ui::node_bundles::NodeBundle};

pub trait SpawnAble {
	fn bundle(colors: BaseColors) -> (NodeBundle, Self);
	fn children(colors: BaseColors, parent: &mut ChildBuilder);
}
