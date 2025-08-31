use crate::{components::KeyedPanel, tools::PanelState};
use bevy::{ecs::component::Mutable, prelude::*};
use common::{
	tools::item_description::ItemToken,
	traits::{
		accessors::set::Setter,
		handles_loadout_menu::GetItem,
		handles_localization::{Localize, Token, localized::Localized},
		inspect_able::{InspectAble, InspectField},
		thread_safe::ThreadSafe,
	},
};
use std::hash::Hash;

impl<T> SetContainerPanels for T where T: Component<Mutability = Mutable> {}

pub trait SetContainerPanels: Component<Mutability = Mutable> + Sized {
	fn set_container_panels<TLocalization, TKey, TEquipment>(
		items: Res<TEquipment>,
		localize: Res<TLocalization>,
		mut texts: Query<(&ChildOf, &mut Text)>,
		mut panels: Query<(Entity, &KeyedPanel<TKey>, &mut Self)>,
	) where
		Self: Setter<PanelState>,
		TLocalization: Localize + Resource,
		TKey: Eq + Hash + Copy + ThreadSafe,
		TEquipment: Resource + GetItem<TKey>,
		TEquipment::TItem: InspectAble<ItemToken>,
	{
		for (entity, KeyedPanel(key), mut panel) in &mut panels {
			let (state, label) = match items.get_item(*key) {
				Some(item) => (
					PanelState::Filled,
					localize.localize(ItemToken::inspect_field(item)).or_token(),
				),
				None => (
					PanelState::Empty,
					localize
						.localize(&Token::from("inventory-item-empty"))
						.or_token(),
				),
			};
			panel.set(state);
			set_label(&mut texts, entity, label);
		}
	}
}

fn set_label(texts: &mut Query<(&ChildOf, &mut Text)>, entity: Entity, label: Localized) {
	let Some((.., mut text)) = texts.iter_mut().find(|(c, ..)| c.parent() == entity) else {
		return;
	};
	*text = Text::from(label);
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::handles_localization::{LocalizationResult, Token, localized::Localized};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashMap;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _Key(usize);

	#[derive(Debug, PartialEq)]
	struct _Item(Token);

	impl From<&str> for _Item {
		fn from(value: &str) -> Self {
			_Item(Token::from(value))
		}
	}

	impl InspectAble<ItemToken> for _Item {
		fn get_inspect_able_field(&self) -> &Token {
			&self.0
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
	impl Localize for _Localize {
		fn localize(&self, token: &Token) -> LocalizationResult {
			self.mock.localize(token)
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
			mock.expect_localize()
				.with(eq(Token::from("inventory-item-empty")))
				.return_const(LocalizationResult::Ok(Localized::from("EMPTY")));
			mock.expect_localize()
				.return_const(LocalizationResult::Error(Token::from("??").failed()));
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
		let text = app
			.world_mut()
			.spawn(Text::new(""))
			.insert(ChildOf(panel))
			.id();

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
			mock.expect_localize()
				.with(eq(Token::from("my item")))
				.return_const(LocalizationResult::Ok(Localized::from(
					"Localized: my item",
				)));
			mock.expect_localize()
				.return_const(LocalizationResult::Error(Token::from("??").failed()));
		});
		let mut app = setup(
			HashMap::from([(_Key(42), _Item::from("my item"))]),
			localize,
		);
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
		let text = app
			.world_mut()
			.spawn(Text::new(""))
			.insert(ChildOf(panel))
			.id();

		app.update();

		assert_eq!(
			Some("Localized: my item"),
			app.world()
				.entity(text)
				.get::<Text>()
				.map(|Text(t)| t.as_str())
		);
	}

	#[test]
	fn still_set_state_when_no_children() {
		let localize = _Localize::new().with_mock(|mock| {
			mock.expect_localize()
				.with(eq(Token::from("my item")))
				.return_const(LocalizationResult::Error(Token::from("??").failed()));
			mock.expect_localize()
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
			mock.expect_localize()
				.with(eq(Token::from("my item")))
				.return_const(LocalizationResult::Ok(Localized::from(
					"Localized: my item",
				)));
			mock.expect_localize()
				.return_const(LocalizationResult::Error(Token::from("??").failed()));
		});
		let mut app = setup(
			HashMap::from([(_Key(42), _Item::from("my item"))]),
			localize,
		);
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
		app.world_mut().spawn(()).insert(ChildOf(panel));
		let text = app
			.world_mut()
			.spawn(Text::new(""))
			.insert(ChildOf(panel))
			.id();

		app.update();

		assert_eq!(
			Some("Localized: my item"),
			app.world()
				.entity(text)
				.get::<Text>()
				.map(|Text(t)| t.as_str())
		);
	}
}
