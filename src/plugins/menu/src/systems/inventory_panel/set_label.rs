use crate::{
	components::{KeyedPanel, inventory_panel::InventoryPanel, label::UILabel},
	tools::PanelState,
};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	traits::{
		accessors::get::{Get, TryApplyOn, TryGetContext, View},
		handles_loadout::items::{Items, ReadItems},
		handles_player::PlayerEntity,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl InventoryPanel {
	pub(crate) fn set_label<TPlayer, TLoadout>(
		mut commands: ZyheedaCommands,
		mut panels: Query<(Entity, &mut Self, &KeyedPanel)>,
		player: StaticSystemParam<TPlayer>,
		param: StaticSystemParam<TLoadout>,
	) where
		TPlayer: for<'w, 's> SystemParam<Item<'w, 's>: View<PlayerEntity>>,
		TLoadout: for<'c> TryGetContext<Items, TContext<'c>: ReadItems>,
	{
		let Some(player) = player.view() else {
			return;
		};
		let Some(entity) = commands.get(&player) else {
			return;
		};
		let Some(ctx) = TLoadout::try_get_context(&param, Items { entity }) else {
			return;
		};

		for (panel_entity, mut panel, KeyedPanel(key)) in &mut panels {
			let panel_state = match ctx.get_item(*key) {
				None => {
					commands.try_apply_on(&panel_entity, |mut e| {
						e.try_insert(UILabel::empty());
					});
					PanelState::Empty
				}
				Some(item) => {
					commands.try_apply_on(&panel_entity, |mut e| {
						e.try_insert(UILabel(item.view().clone()));
					});
					PanelState::Filled
				}
			};
			*panel = Self(panel_state);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{KeyedPanel, label::UILabel},
		testing::{_Player, _PlayerParam},
		tools::PanelState,
	};
	use common::{
		CommonPlugin,
		tools::action_key::slot::HandSlot,
		traits::{
			handles_loadout::{LoadoutKey, items::ItemToken},
			handles_localization::Token,
		},
	};
	use std::{collections::HashMap, sync::LazyLock};
	use testing::SingleThreadedApp;

	#[derive(Clone)]
	struct _Item(Token);

	impl View<ItemToken> for _Item {
		fn view(&self) -> &Token {
			&self.0
		}
	}

	#[derive(Component, Default)]
	struct _Container(HashMap<LoadoutKey, _Item>);

	impl<T, TKey> From<T> for _Container
	where
		T: IntoIterator<Item = (TKey, _Item)>,
		TKey: Into<LoadoutKey>,
	{
		fn from(value: T) -> Self {
			Self(HashMap::from_iter(
				value.into_iter().map(|(k, i)| (k.into(), i)),
			))
		}
	}

	impl ReadItems for _Container {
		type TItem<'a>
			= _Item
		where
			Self: 'a;

		fn get_item<TKey>(&self, key: TKey) -> Option<Self::TItem<'_>>
		where
			TKey: Into<LoadoutKey>,
		{
			self.0.get(&key.into()).cloned()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(CommonPlugin::with_asset_loading(false));
		app.add_systems(
			Update,
			InventoryPanel::set_label::<_PlayerParam, Query<Ref<_Container>>>,
		);

		app
	}

	static TOKEN: LazyLock<Token> = LazyLock::new(|| Token::from("my token"));

	#[test]
	fn set_label() {
		let mut app = setup();
		app.world_mut().spawn((
			_Player,
			_Container::from([(HandSlot::Left, _Item(TOKEN.clone()))]),
		));
		let panel = app
			.world_mut()
			.spawn((
				InventoryPanel(PanelState::Empty),
				KeyedPanel::from(HandSlot::Left),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&UILabel(Token::from("my token"))),
			app.world().entity(panel).get::<UILabel<Token>>(),
		);
	}

	#[test]
	fn set_panel_to_filled() {
		let mut app = setup();
		app.world_mut().spawn((
			_Player,
			_Container::from([(HandSlot::Left, _Item(TOKEN.clone()))]),
		));
		let panel = app
			.world_mut()
			.spawn((
				InventoryPanel(PanelState::Empty),
				KeyedPanel::from(HandSlot::Left),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&InventoryPanel(PanelState::Filled)),
			app.world().entity(panel).get::<InventoryPanel>(),
		);
	}

	#[test]
	fn set_panel_to_empty() {
		let mut app = setup();
		app.world_mut().spawn((_Player, _Container::default()));
		let panel = app
			.world_mut()
			.spawn((
				InventoryPanel(PanelState::Filled),
				KeyedPanel::from(HandSlot::Left),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&InventoryPanel(PanelState::Empty)),
			app.world().entity(panel).get::<InventoryPanel>(),
		);
	}

	#[test]
	fn do_nothing_if_agent_missing() {
		let mut app = setup();
		app.world_mut()
			.spawn(_Container::from([(HandSlot::Left, _Item(TOKEN.clone()))]));
		let panel = app
			.world_mut()
			.spawn((
				InventoryPanel(PanelState::Empty),
				KeyedPanel::from(HandSlot::Left),
			))
			.id();

		app.update();

		let panel = app.world().entity(panel);
		assert_eq!(
			(None, Some(&InventoryPanel(PanelState::Empty))),
			(panel.get::<UILabel<Token>>(), panel.get::<InventoryPanel>()),
		);
	}
}
