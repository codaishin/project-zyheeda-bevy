pub(crate) mod menu_state;

use bevy::ui::{UiRect, Val};
use common::tools::Index;

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
	pub const SINGLE_COLUMN: Layout = Layout::LastColumn(Index(0));
	pub const SINGLE_ROW: Layout = Layout::LastRow(Index(0));
}

impl Default for Layout {
	fn default() -> Self {
		Self::SINGLE_COLUMN
	}
}

#[derive(Default)]
pub(crate) struct Pixel(pub f32);

impl From<Pixel> for Val {
	fn from(value: Pixel) -> Self {
		Val::Px(value.0)
	}
}

impl From<Pixel> for UiRect {
	fn from(value: Pixel) -> Self {
		UiRect::all(Val::from(value))
	}
}

#[derive(Default)]
pub(crate) struct Dimensions {
	pub(crate) width: Pixel,
	pub(crate) height: Pixel,
	pub(crate) border: Pixel,
}

impl Dimensions {
	pub(crate) fn nested_height(&self) -> Pixel {
		Pixel(self.height.0 - self.border.0)
	}

	pub(crate) fn nested_width(&self) -> Pixel {
		Pixel(self.width.0 - self.border.0)
	}

	pub(crate) fn nested_minimum(&self) -> Pixel {
		Pixel(-self.border.0)
	}
}
