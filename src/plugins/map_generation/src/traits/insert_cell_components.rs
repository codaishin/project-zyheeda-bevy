use bevy::prelude::*;

pub(crate) trait InsertCellComponents {
	fn offset_height(&self) -> bool;
	fn insert_cell_components(&self, entity: &mut EntityCommands);
}
