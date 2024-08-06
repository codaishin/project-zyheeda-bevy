#![allow(dead_code)] // FIXME: remove when `Layout::LastRow(..)` and `Layout::SINGLE_ROW` is used

pub(crate) mod menu_state;

use bevy::ui::Val;
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

pub(crate) struct Pixel(pub f32);

impl From<Pixel> for Val {
	fn from(value: Pixel) -> Self {
		Val::Px(value.0)
	}
}

pub(crate) struct Dimensions<T> {
	pub width: T,
	pub height: T,
}
