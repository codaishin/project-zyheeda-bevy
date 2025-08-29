use crate::{
	components::{KeyedPanel, inventory_panel::InventoryPanel, label::UILabel},
	tools::PanelState,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetParamEntry, Param, ParamEntry, RefAs, RefInto, TryApplyOn},
		handles_loadout::{ContainerKey, ItemToken},
	},
	zyheeda_commands::ZyheedaCommands,
};

impl InventoryPanel {
	pub(crate) fn set_label<TAgent, TContainer>(
		mut commands: ZyheedaCommands,
		containers: Query<&TContainer, With<TAgent>>,
		mut panels: Query<(Entity, &mut Self, &KeyedPanel<TContainer::TKey>)>,
		param: StaticSystemParam<Param<TContainer, TContainer::TKey>>,
	) where
		TAgent: Component,
		for<'w, 's> TContainer: Component + ContainerKey + GetParamEntry<'w, 's, TContainer::TKey>,
		for<'w, 's, 'a> ParamEntry<'w, 's, TContainer, TContainer::TKey>:
			RefInto<'a, Option<ItemToken<'a>>>,
	{
		for container in &containers {
			for (entity, mut panel, KeyedPanel(key)) in &mut panels {
				let item = container.get_param_entry(key, &param);
				let panel_state = match item.ref_as::<Option<ItemToken>>() {
					None => {
						commands.try_apply_on(&entity, |mut e| {
							e.try_insert(UILabel::empty());
						});
						PanelState::Empty
					}
					Some(ItemToken(token)) => {
						commands.try_apply_on(&entity, |mut e| {
							e.try_insert(UILabel(token.clone()));
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
	use common::traits::handles_localization::{Token, localized::Localized};
	use std::sync::LazyLock;
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Agent;

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _Key;

	#[derive(Clone)]
	struct _Entry(Option<ItemToken<'static>>);

	impl<'a> From<&'a _Entry> for Option<ItemToken<'a>> {
		fn from(_Entry(token): &'a _Entry) -> Self {
			token.clone()
		}
	}

	#[derive(Component)]
	struct _Container(_Entry);

	impl ContainerKey for _Container {
		type TKey = _Key;
	}

	impl GetParamEntry<'_, '_, _Key> for _Container {
		type TParam = ();
		type TEntry = _Entry;

		fn get_param_entry(&self, _: &_Key, _: &()) -> Self::TEntry {
			self.0.clone()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, InventoryPanel::set_label::<_Agent, _Container>);

		app
	}

	static TOKEN: LazyLock<Token> = LazyLock::new(|| Token::from("my token"));

	#[test]
	fn set_label() {
		let mut app = setup();
		app.world_mut()
			.spawn((_Agent, _Container(_Entry(Some(ItemToken(&TOKEN))))));
		let panel = app
			.world_mut()
			.spawn((InventoryPanel(PanelState::Empty), KeyedPanel(_Key)))
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
		app.world_mut()
			.spawn((_Agent, _Container(_Entry(Some(ItemToken(&TOKEN))))));
		let panel = app
			.world_mut()
			.spawn((InventoryPanel(PanelState::Empty), KeyedPanel(_Key)))
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
		app.world_mut().spawn((_Agent, _Container(_Entry(None))));
		let panel = app
			.world_mut()
			.spawn((InventoryPanel(PanelState::Filled), KeyedPanel(_Key)))
			.id();

		app.update();

		assert_eq!(
			Some(&InventoryPanel(PanelState::Empty)),
			app.world().entity(panel).get::<InventoryPanel>(),
		);
	}

	#[test]
	fn empty_label() {
		let mut app = setup();
		app.world_mut().spawn((_Agent, _Container(_Entry(None))));
		let panel = app
			.world_mut()
			.spawn((
				InventoryPanel(PanelState::Filled),
				UILabel(Localized::from("something")),
				KeyedPanel(_Key),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&UILabel::empty()),
			app.world().entity(panel).get::<UILabel>(),
		);
	}

	#[test]
	fn do_nothing_if_agent_missing() {
		let mut app = setup();
		app.world_mut()
			.spawn(_Container(_Entry(Some(ItemToken(&TOKEN)))));
		let panel = app
			.world_mut()
			.spawn((InventoryPanel(PanelState::Empty), KeyedPanel(_Key)))
			.id();

		app.update();

		let panel = app.world().entity(panel);
		assert_eq!(
			(None, Some(&InventoryPanel(PanelState::Empty))),
			(panel.get::<UILabel<Token>>(), panel.get::<InventoryPanel>()),
		);
	}
}
