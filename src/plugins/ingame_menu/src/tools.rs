#![allow(dead_code)] // FIXME: remove when `Layout::LastRow(..)` and `Layout::SINGLE_ROW` is used

pub(crate) mod menu_state;

use common::tools::Index;
use skills::skills::Skill;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct SkillDescriptor<TKey> {
	pub(crate) key_path: Vec<TKey>,
	pub(crate) skill: Skill,
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
