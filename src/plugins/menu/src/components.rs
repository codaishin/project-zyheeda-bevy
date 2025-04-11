pub(crate) mod button_interaction;
pub(crate) mod combo_overview;
pub(crate) mod combo_skill_button;
pub(crate) mod dropdown;
pub(crate) mod inventory_panel;
pub(crate) mod inventory_screen;
pub(crate) mod key_code_text_insert_command;
pub(crate) mod key_select;
pub(crate) mod key_select_dropdown_command;
pub(crate) mod loading_screen;
pub(crate) mod quickbar_panel;
pub(crate) mod start_game;
pub(crate) mod start_menu;
pub(crate) mod start_menu_button;
pub(crate) mod tooltip;
pub(crate) mod ui_overlay;

use bevy::prelude::*;
use combo_skill_button::Horizontal;
use common::tools::keys::slot::SlotKey;
use std::marker::PhantomData;

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub struct Dad<T>(pub T);

#[derive(Component, Debug, Clone, Copy)]
pub struct KeyedPanel<TKey>(pub TKey);

#[derive(Component)]
pub struct Label<T, TKey> {
	pub key: TKey,
	phantom_data: PhantomData<T>,
}

impl<T, TKey> Label<T, TKey> {
	pub fn new(key: TKey) -> Self {
		Self {
			key,
			phantom_data: PhantomData,
		}
	}
}

#[derive(Component)]
pub struct Quickbar;

#[derive(Component)]
pub struct ColorOverride;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct SkillSelectDropdownInsertCommand<TKey = SlotKey, TLayout = Horizontal> {
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
	pub(crate) key_path: Vec<SlotKey>,
}

#[derive(Component, Debug, PartialEq)]
pub(crate) struct ImageColorCommand(pub(crate) Color);

#[derive(Component, Debug, PartialEq)]
pub(crate) struct GlobalZIndexTop;
