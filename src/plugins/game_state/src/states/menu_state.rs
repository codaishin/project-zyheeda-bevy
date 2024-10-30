#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
pub enum MenuState {
	#[default]
	Inventory,
	ComboOverview,
}
