use super::menu_background::MenuBackground;
use crate::{
	LoadUi,
	traits::{
		colors::DEFAULT_PANEL_COLORS,
		insert_ui_content::InsertUiContent,
		update_key_bindings::UpdateKeyBindings,
	},
};
use bevy::prelude::*;
use common::{
	tools::keys::{
		Key,
		movement::MovementKey,
		slot::{Side, SlotKey},
	},
	traits::{
		handles_localization::{LocalizeToken, localized::Localized},
		iterate::Iterate,
		thread_safe::ThreadSafe,
	},
};
use std::collections::HashMap;

#[derive(Component, Debug, PartialEq, Default)]
#[require(MenuBackground)]
pub(crate) struct SettingsScreen {
	key_bindings: HashMap<Key, KeyCode>,
}

impl LoadUi<AssetServer> for SettingsScreen {
	fn load_ui(_: &mut AssetServer) -> Self {
		Self::default()
	}
}

const SLOT_KEYS: &[Key] = &[
	Key::Slot(SlotKey::TopHand(Side::Left)),
	Key::Slot(SlotKey::BottomHand(Side::Left)),
	Key::Slot(SlotKey::TopHand(Side::Right)),
	Key::Slot(SlotKey::BottomHand(Side::Right)),
];

const MOVEMENT_KEYS: &[Key] = &[
	Key::Movement(MovementKey::Forward),
	Key::Movement(MovementKey::Backward),
	Key::Movement(MovementKey::Left),
	Key::Movement(MovementKey::Right),
];

impl InsertUiContent for SettingsScreen {
	fn insert_ui_content<TLocalization>(
		&self,
		localize: &mut TLocalization,
		parent: &mut ChildBuilder,
	) where
		TLocalization: LocalizeToken + ThreadSafe,
	{
		parent
			.spawn(Node {
				flex_direction: FlexDirection::Column,
				align_items: AlignItems::Center,
				..default()
			})
			.with_children(|parent| {
				add_title(parent, localize.localize_token("key-bindings").or_token());
				for key in SLOT_KEYS {
					let Some(key_code) = self.key_bindings.get(key) else {
						continue;
					};
					add_key_row(parent, localize, key, key_code);
				}
				for key in MOVEMENT_KEYS {
					let Some(key_code) = self.key_bindings.get(key) else {
						continue;
					};
					add_key_row(parent, localize, key, key_code);
				}
			});
	}
}

fn add_title(parent: &mut ChildBuilder, title: Localized) {
	parent.spawn((
		Text::new(title),
		TextFont {
			font_size: 40.0,
			..default()
		},
		TextColor(DEFAULT_PANEL_COLORS.text),
	));
}

fn add_key_row<TLocalization>(
	parent: &mut ChildBuilder,
	localize: &mut TLocalization,
	key: &Key,
	key_code: &KeyCode,
) where
	TLocalization: LocalizeToken + ThreadSafe,
{
	parent
		.spawn(Node {
			flex_direction: FlexDirection::Row,
			..default()
		})
		.with_children(|parent| {
			parent.spawn(Text::from(localize.localize_token(*key).or_token()));
			parent.spawn(Text::from(localize.localize_token(*key_code).or_token()));
		});
}

impl UpdateKeyBindings<Key, KeyCode> for SettingsScreen {
	fn update_key_bindings<TKeyMap>(&mut self, map: &TKeyMap)
	where
		for<'a> TKeyMap: Iterate<'a, TItem = (&'a Key, &'a KeyCode)>,
	{
		self.key_bindings = map
			.iterate()
			.map(|(key, key_code)| (*key, *key_code))
			.collect()
	}
}
