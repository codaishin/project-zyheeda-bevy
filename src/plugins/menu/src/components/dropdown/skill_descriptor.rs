use std::fmt::Debug;

use super::Dropdown;
use crate::{
	components::{
		combo_overview::ComboOverview,
		combo_skill_button::{ComboSkillButton, DropdownItem, Horizontal, Vertical},
	},
	tools::Layout,
	traits::{GetLayout, GetRootNode},
};
use bevy::prelude::*;
use common::traits::thread_safe::ThreadSafe;

impl<TId> GetRootNode for Dropdown<ComboSkillButton<DropdownItem<Vertical>, TId>>
where
	TId: Debug + PartialEq + ThreadSafe + Clone,
{
	fn root_node(&self) -> Node {
		Node {
			position_type: PositionType::Absolute,
			top: Val::from(ComboOverview::SKILL_BUTTON_DIMENSIONS.height_inner()),
			left: Val::from(ComboOverview::SKILL_BUTTON_DIMENSIONS.minimum_inner()),
			..default()
		}
	}
}

impl<TId> GetLayout for Dropdown<ComboSkillButton<DropdownItem<Vertical>, TId>>
where
	TId: Debug + PartialEq + ThreadSafe + Clone,
{
	fn layout(&self) -> Layout {
		Layout::SINGLE_COLUMN
	}
}

impl<TId> GetRootNode for Dropdown<ComboSkillButton<DropdownItem<Horizontal>, TId>>
where
	TId: Debug + PartialEq + ThreadSafe + Clone,
{
	fn root_node(&self) -> Node {
		Node {
			position_type: PositionType::Absolute,
			top: Val::from(ComboOverview::KEY_BUTTON_DIMENSIONS.minimum_inner()),
			left: Val::from(ComboOverview::KEY_BUTTON_DIMENSIONS.width_inner()),
			..default()
		}
	}
}

impl<TId> GetLayout for Dropdown<ComboSkillButton<DropdownItem<Horizontal>, TId>>
where
	TId: Debug + PartialEq + ThreadSafe + Clone,
{
	fn layout(&self) -> Layout {
		Layout::SINGLE_ROW
	}
}
