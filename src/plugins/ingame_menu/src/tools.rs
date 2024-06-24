pub(crate) mod menu_state;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PanelState {
	Empty,
	Filled,
}
