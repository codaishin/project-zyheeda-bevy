use common::zyheeda_commands::ZyheedaEntityCommands;

pub(crate) trait InsertCellComponents {
	fn offset_height(&self) -> bool;
	fn insert_cell_components(&self, entity: &mut ZyheedaEntityCommands);
}
