use common::tools::Index;

pub(crate) mod menu_state;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PanelState {
	Empty,
	Filled,
}

pub enum Layout {
	MaxColumn(Index),
	MaxRow(Index),
}

impl Layout {
	pub(crate) const DEFAULT: Layout = Layout::single_column();

	pub const fn single_column() -> Self {
		Self::MaxColumn(Index(1))
	}

	pub const fn single_row() -> Self {
		Self::MaxRow(Index(1))
	}
}

impl Default for Layout {
	fn default() -> Self {
		Self::DEFAULT
	}
}
