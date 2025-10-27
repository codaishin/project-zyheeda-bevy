use crate::{
	components::{KeyedPanel, inventory_panel::InventoryPanel, label::UILabel},
	tools::PanelState,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{EntityContext, GetProperty, TryApplyOn},
		handles_loadout::items::{Items, ReadItems},
	},
	zyheeda_commands::ZyheedaCommands,
};

impl InventoryPanel {
	pub(crate) fn set_label<TAgent, TLoadout>(
		mut commands: ZyheedaCommands,
		agents: Query<Entity, With<TAgent>>,
		mut panels: Query<(Entity, &mut Self, &KeyedPanel)>,
		param: StaticSystemParam<TLoadout>,
	) where
		TAgent: Component,
		TLoadout: for<'c> EntityContext<Items, TContext<'c>: ReadItems>,
	{
		for agent in &agents {
			let Some(ctx) = TLoadout::get_entity_context(&param, agent, Items) else {
				continue;
			};

			for (entity, mut panel, KeyedPanel(key)) in &mut panels {
				let panel_state = match ctx.get_item(*key) {
					None => {
						commands.try_apply_on(&entity, |mut e| {
							e.try_insert(UILabel::empty());
						});
						PanelState::Empty
					}
					Some(item) => {
						commands.try_apply_on(&entity, |mut e| {
							e.try_insert(UILabel(item.get_property().clone()));
						});
						PanelState::Filled
					}
				};
				*panel = Self(panel_state);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{KeyedPanel, label::UILabel},
		tools::PanelState,
	};
	use common::{
		tools::action_key::slot::PlayerSlot,
		traits::{
			handles_loadout::{LoadoutKey, items::ItemToken},
			handles_localization::Token,
		},
	};
	use std::{collections::HashMap, sync::LazyLock};
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Agent;

	#[derive(Clone)]
	struct _Item(Token);

	impl GetProperty<ItemToken> for _Item {
		fn get_property(&self) -> &Token {
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

		app.add_systems(
			Update,
			InventoryPanel::set_label::<_Agent, Query<Ref<_Container>>>,
		);

		app
	}

	static TOKEN: LazyLock<Token> = LazyLock::new(|| Token::from("my token"));

	#[test]
	fn set_label() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Container::from([(PlayerSlot::UPPER_L, _Item(TOKEN.clone()))]),
		));
		let panel = app
			.world_mut()
			.spawn((
				InventoryPanel(PanelState::Empty),
				KeyedPanel::from(PlayerSlot::UPPER_L),
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
			_Agent,
			_Container::from([(PlayerSlot::UPPER_L, _Item(TOKEN.clone()))]),
		));
		let panel = app
			.world_mut()
			.spawn((
				InventoryPanel(PanelState::Empty),
				KeyedPanel::from(PlayerSlot::UPPER_L),
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
		app.world_mut().spawn((_Agent, _Container::default()));
		let panel = app
			.world_mut()
			.spawn((
				InventoryPanel(PanelState::Filled),
				KeyedPanel::from(PlayerSlot::UPPER_L),
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
		app.world_mut().spawn(_Container::from([(
			PlayerSlot::UPPER_L,
			_Item(TOKEN.clone()),
		)]));
		let panel = app
			.world_mut()
			.spawn((
				InventoryPanel(PanelState::Empty),
				KeyedPanel::from(PlayerSlot::UPPER_L),
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
