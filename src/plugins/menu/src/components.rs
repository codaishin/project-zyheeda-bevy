pub(crate) mod button_interaction;
pub(crate) mod combo_overview;
pub(crate) mod combo_skill_button;
pub(crate) mod dispatch_text_color;
pub(crate) mod dropdown;
pub(crate) mod icon;
pub(crate) mod input_label;
pub(crate) mod inventory_panel;
pub(crate) mod inventory_screen;
pub(crate) mod key_select;
pub(crate) mod key_select_dropdown_command;
pub(crate) mod loading_screen;
pub(crate) mod menu_background;
pub(crate) mod prevent_menu_change;
pub(crate) mod quickbar_panel;
pub(crate) mod settings_screen;
pub(crate) mod start_menu;
pub(crate) mod start_menu_button;
pub(crate) mod tooltip;
pub(crate) mod ui_disabled;
pub(crate) mod ui_overlay;

use bevy::prelude::*;
use combo_skill_button::Horizontal;
use common::tools::action_key::slot::PlayerSlot;
use std::marker::PhantomData;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub struct Dad<T>(pub T);

#[derive(Component, Debug, Clone, Copy)]
pub struct KeyedPanel<TKey>(pub TKey);

#[derive(Component)]
pub struct Quickbar;

#[derive(Component)]
pub struct ColorOverride;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct SkillSelectDropdownInsertCommand<TKey = PlayerSlot, TLayout = Horizontal> {
	phantom_data: PhantomData<TLayout>,
	pub(crate) key_path: Vec<TKey>,
}

impl<TKey, TLayout> SkillSelectDropdownInsertCommand<TKey, TLayout> {
	pub(crate) fn new(key_path: Vec<TKey>) -> Self {
		Self {
			phantom_data: PhantomData,
			key_path,
		}
	}
}

#[derive(Component, Debug, PartialEq)]
pub(crate) struct DeleteSkill {
	pub(crate) key_path: Vec<PlayerSlot>,
}

#[derive(Component, Debug, PartialEq)]
pub(crate) struct ImageColorCommand(pub(crate) Color);

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct GlobalZIndexTop;
