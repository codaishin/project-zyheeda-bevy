use common::tools::Index;

pub(crate) mod menu_state;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PanelState {
	Empty,
	Filled,
}

pub enum Layout {
	LastColumn(Index<u16>),
	LastRow(Index<u16>),
}

impl Layout {
	pub(crate) const DEFAULT: Layout = Layout::single_column();

	pub const fn single_column() -> Self {
		Self::LastColumn(Index(0))
	}

	pub const fn single_row() -> Self {
		Self::LastRow(Index(0))
	}
}

impl Default for Layout {
	fn default() -> Self {
		Self::DEFAULT
	}
}
