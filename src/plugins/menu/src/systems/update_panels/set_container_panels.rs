use crate::{components::KeyedPanel, tools::PanelState};
use bevy::{hierarchy::Parent, prelude::*};
use common::{
	tools::item_description::ItemDescription,
	traits::{
		accessors::set::Setter,
		handles_loadout_menu::GetItem,
		handles_localization::LocalizeToken,
		inspect_able::{InspectAble, InspectField},
		thread_safe::ThreadSafe,
	},
};
use std::hash::Hash;

impl<T> SetContainerPanels for T {}

pub trait SetContainerPanels {
	fn set_container_panels<TLocalization, TKey, TEquipment>(
		items: Res<TEquipment>,
		mut localize: ResMut<TLocalization>,
		mut texts: Query<(&Parent, &mut Text)>,
		mut panels: Query<(Entity, &KeyedPanel<TKey>, &mut Self)>,
	) where
		Self: Component + Setter<PanelState> + Sized,
		TLocalization: LocalizeToken + Resource,
		TKey: Eq + Hash + Copy + ThreadSafe,
		TEquipment: Resource + GetItem<TKey>,
		TEquipment::TItem: InspectAble<ItemDescription>,
	{
		for (entity, KeyedPanel(key), mut panel) in &mut panels {
			let (state, label) = match items.get_item(*key) {
				Some(item) => (PanelState::Filled, ItemDescription::inspect_field(item)),
				None => (
					PanelState::Empty,
					String::from(localize.localize_token("inventory-item-empty").or_token()),
				),
			};
			panel.set(state);
			set_label(&mut texts, entity, label);
		}
	}
}

fn set_label(texts: &mut Query<(&Parent, &mut Text)>, entity: Entity, label: String) {
	let Some((.., mut text)) = texts.iter_mut().find(|(p, ..)| p.get() == entity) else {
		return;
	};
	let Text(text) = text.as_mut();

	*text = label;
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::{
			handles_localization::{LocalizationResult, Token, localized::Localized},
			nested_mock::NestedMocks,
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashMap;

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _Key(usize);

	#[derive(Debug, PartialEq)]
	struct _Item(&'static str);

	impl InspectAble<ItemDescription> for _Item {
		fn get_inspect_able_field(&self) -> String {
			self.0.to_owned()
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _ItemDescriptors(HashMap<_Key, _Item>);

	impl GetItem<_Key> for _ItemDescriptors {
		type TItem = _Item;

		fn get_item(&self, key: _Key) -> Option<&_Item> {
			self.0.get(&key)
		}
	}

	#[derive(Component, NestedMocks)]
	struct _Panel {
		mock: Mock_Panel,
	}

	#[automock]
	impl Setter<PanelState> for _Panel {
		fn set(&mut self, value: PanelState) {
			self.mock.set(value)
		}
	}

	#[derive(Resource, NestedMocks)]
	struct _Localize {
		mock: Mock_Localize,
	}

	#[automock]
	impl LocalizeToken for _Localize {
		fn localize_token<TToken>(&mut self, token: TToken) -> LocalizationResult
		where
			TToken: Into<Token> + 'static,
		{
			self.mock.localize_token(token)
		}
	}

	fn setup(descriptors: HashMap<_Key, _Item>, localize: _Localize) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(localize);
		app.insert_resource(_ItemDescriptors(descriptors));
		app.add_systems(
			Update,
			_Panel::set_container_panels::<_Localize, _Key, _ItemDescriptors>,
		);

		app
	}

	#[test]
	fn set_empty() {
		let localize = _Localize::new().with_mock(|mock| {
			mock.expect_localize_token::<&str>()
				.with(eq("inventory-item-empty"))
				.return_const(LocalizationResult::Ok(Localized::from("EMPTY")));
		});
		let mut app = setup(HashMap::default(), localize);
		let panel = app
			.world_mut()
			.spawn((
				KeyedPanel(_Key(42)),
				_Panel::new().with_mock(|mock| {
					mock.expect_set()
						.times(1)
						.with(eq(PanelState::Empty))
						.return_const(());
				}),
			))
			.id();
		let text = app.world_mut().spawn(Text::new("")).set_parent(panel).id();

		app.update();

		assert_eq!(
			Some("EMPTY"),
			app.world()
				.entity(text)
				.get::<Text>()
				.map(|Text(t)| t.as_str())
		);
	}

	#[test]
	fn set_filled() {
		let localize = _Localize::new().with_mock(|mock| {
			mock.expect_localize_token::<&str>()
				.return_const(LocalizationResult::Error(Token::from("??").failed()));
		});
		let mut app = setup(HashMap::from([(_Key(42), _Item("my item"))]), localize);
		let panel = app
			.world_mut()
			.spawn((
				KeyedPanel(_Key(42)),
				_Panel::new().with_mock(|mock| {
					mock.expect_set()
						.times(1)
						.with(eq(PanelState::Filled))
						.return_const(());
				}),
			))
			.id();
		let text = app.world_mut().spawn(Text::new("")).set_parent(panel).id();

		app.update();

		assert_eq!(
			Some("my item"),
			app.world()
				.entity(text)
				.get::<Text>()
				.map(|Text(t)| t.as_str())
		);
	}

	#[test]
	fn still_set_state_when_no_children() {
		let localize = _Localize::new().with_mock(|mock| {
			mock.expect_localize_token::<&str>()
				.return_const(LocalizationResult::Error(Token::from("??").failed()));
		});
		let mut app = setup(HashMap::default(), localize);
		app.world_mut().spawn((
			KeyedPanel(_Key(42)),
			_Panel::new().with_mock(|mock| {
				mock.expect_set().times(1).return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn set_when_text_not_first_child() {
		let localize = _Localize::new().with_mock(|mock| {
			mock.expect_localize_token::<&str>()
				.return_const(LocalizationResult::Error(Token::from("??").failed()));
		});
		let mut app = setup(HashMap::from([(_Key(42), _Item("my item"))]), localize);
		let panel = app
			.world_mut()
			.spawn((
				KeyedPanel(_Key(42)),
				_Panel::new().with_mock(|mock| {
					mock.expect_set()
						.times(1)
						.with(eq(PanelState::Filled))
						.return_const(());
				}),
			))
			.id();
		app.world_mut().spawn(()).set_parent(panel);
		let text = app.world_mut().spawn(Text::new("")).set_parent(panel).id();

		app.update();

		assert_eq!(
			Some("my item"),
			app.world()
				.entity(text)
				.get::<Text>()
				.map(|Text(t)| t.as_str())
		);
	}
}
