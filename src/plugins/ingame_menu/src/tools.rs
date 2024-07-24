#![allow(dead_code)] // FIXME: remove when `Layout::LastRow(..)` and `Layout::SINGLE_ROW` is used

pub(crate) mod menu_state;

use crate::traits::colors::DEFAULT_PANEL_COLORS;
use bevy::{
	prelude::{default, NodeBundle, TextBundle},
	text::TextStyle,
	ui::{Style, UiRect, Val, ZIndex},
};
use common::tools::Index;

pub(crate) fn skill_node() -> NodeBundle {
	NodeBundle {
		style: Style {
			top: Val::Px(-25.0),
			padding: UiRect::all(Val::Px(5.0)),
			..default()
		},
		background_color: DEFAULT_PANEL_COLORS.text.into(),
		z_index: ZIndex::Global(1),
		..default()
	}
}

pub(crate) fn skill_name(name: &str) -> TextBundle {
	TextBundle::from_section(
		name,
		TextStyle {
			font_size: 20.0,
			color: DEFAULT_PANEL_COLORS.filled,
			..default()
		},
	)
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct SkillDescriptor<TKey, TIcon: Clone> {
	pub name: String,
	pub key_path: Vec<TKey>,
	pub icon: Option<TIcon>,
}

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
