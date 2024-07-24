pub(crate) mod combo_overview;
pub(crate) mod dropdown;
pub(crate) mod inventory_panel;
pub(crate) mod inventory_screen;
pub(crate) mod quickbar_panel;
pub(crate) mod skill_select;
pub(crate) mod tooltip;
pub(crate) mod ui_overlay;

use bevy::ecs::component::Component;
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
pub(crate) struct SkillSelectDropdownCommand<TKey> {
	pub(crate) key_path: Vec<TKey>,
}
