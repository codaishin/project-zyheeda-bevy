use super::{GetLayout, GetRootNode, insert_ui_content::InsertUiContent};
use crate::{
	Dropdown,
	systems::dropdown::{
		despawn_when_no_children_pressed::dropdown_despawn_when_no_children_pressed,
		detect_focus_change::dropdown_detect_focus_change,
		events::dropdown_events,
		spawn_focused::dropdown_spawn_focused,
		track_child_dropdowns::dropdown_track_child_dropdowns,
	},
};
use bevy::prelude::*;
use common::traits::handles_localization::LocalizeToken;

pub(crate) trait AddDropdown {
	fn add_dropdown<TLocalization, TItem>(&mut self) -> &mut Self
	where
		TLocalization: LocalizeToken + Resource,
		TItem: InsertUiContent + Sync + Send + 'static,
		Dropdown<TItem>: GetRootNode + GetLayout;
}

impl AddDropdown for App {
	fn add_dropdown<TLocalization, TItem>(&mut self) -> &mut Self
	where
		TLocalization: LocalizeToken + Resource,
		TItem: InsertUiContent + Sync + Send + 'static,
		Dropdown<TItem>: GetRootNode + GetLayout,
	{
		self.add_systems(
			Update,
			(
				dropdown_events::<TItem>,
				dropdown_track_child_dropdowns::<TItem>,
			),
		)
		.add_systems(
			Last,
			dropdown_detect_focus_change::<TItem>
				.pipe(dropdown_despawn_when_no_children_pressed::<TItem>)
				.pipe(dropdown_spawn_focused::<TLocalization, TItem>),
		)
	}
}
