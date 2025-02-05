use crate::{components::KeyedPanel, tools::PanelState};
use bevy::{hierarchy::Parent, prelude::*};
use common::traits::{
	accessors::{
		get::{GetField, Getter},
		set::Setter,
	},
	handles_equipment::{ItemName, SingleAccess},
};
use std::fmt::Debug;

impl<T> SetContainerPanels for T {}

pub trait SetContainerPanels {
	fn set_container_panels<TPanel, TKey>(
		containers: Query<&Self>,
		mut texts: Query<(&Parent, &mut Text)>,
		mut panels: Query<(Entity, &KeyedPanel<TKey>, &mut TPanel)>,
		items: Res<Assets<Self::TItem>>,
	) where
		Self: Component + SingleAccess<TKey = TKey> + Sized,
		Self::TItem: Asset + Getter<ItemName>,
		TPanel: Component + Setter<PanelState>,
		TKey: Debug + Copy + Send + Sync + 'static,
	{
		let container = containers.single();

		for (entity, KeyedPanel(key), mut panel) in &mut panels {
			let (state, ItemName(label)) = match get_item(container, key, &items) {
				Some(item) => (PanelState::Filled, ItemName::get_field(item)),
				None => (PanelState::Empty, ItemName("<Empty>".to_owned())),
			};
			panel.set(state);
			set_label(&mut texts, entity, label);
		}
	}
}

fn get_item<'a, TContainer, TKey>(
	container: &'a TContainer,
	key: &TKey,
	items: &'a Assets<TContainer::TItem>,
) -> Option<&'a TContainer::TItem>
where
	TContainer: SingleAccess<TKey = TKey>,
	TContainer::TItem: Asset,
{
	container
		.single_access(key)
		.ok()
		.and_then(|handle| handle.as_ref())
		.and_then(|handle| items.get(handle))
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
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::traits::{handles_equipment::KeyOutOfBounds, nested_mock::NestedMocks};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashMap;

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _Key(usize);

	#[derive(Asset, TypePath, Debug, PartialEq)]
	struct _Item(&'static str);

	impl Getter<ItemName> for _Item {
		fn get(&self) -> ItemName {
			ItemName(self.0.to_owned())
		}
	}

	#[derive(Component)]
	struct _Container(HashMap<_Key, Option<Handle<_Item>>>);

	impl SingleAccess for _Container {
		type TKey = _Key;
		type TItem = _Item;

		fn single_access(
			&self,
			key: &Self::TKey,
		) -> Result<&Option<Handle<Self::TItem>>, KeyOutOfBounds> {
			let Some(item) = self.0.get(key) else {
				return Err(KeyOutOfBounds);
			};

			Ok(item)
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

	fn setup_container<const N: usize>(items: [(_Key, _Item); N]) -> (_Container, Assets<_Item>) {
		let mut item_assets = Assets::default();
		let mut container = HashMap::default();

		for (key, item) in items {
			let item = item_assets.add(item);
			container.insert(key, Some(item));
		}

		(_Container(container), item_assets)
	}

	fn setup_app<const N: usize>(items: [(_Key, _Item); N]) -> App {
		let (container, items) = setup_container(items);
		let mut app = App::new();
		app.insert_resource(items);
		app.world_mut().spawn(container);

		app
	}

	#[test]
	fn set_empty() -> Result<(), RunSystemError> {
		let mut app = setup_app([]);
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

		app.world_mut()
			.run_system_once(_Container::set_container_panels::<_Panel, _Key>)?;

		assert_eq!(
			Some("<Empty>"),
			app.world()
				.entity(text)
				.get::<Text>()
				.map(|Text(t)| t.as_str())
		);
		Ok(())
	}

	#[test]
	fn set_filled() -> Result<(), RunSystemError> {
		let mut app = setup_app([(_Key(42), _Item("my item"))]);
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

		app.world_mut()
			.run_system_once(_Container::set_container_panels::<_Panel, _Key>)?;

		assert_eq!(
			Some("my item"),
			app.world()
				.entity(text)
				.get::<Text>()
				.map(|Text(t)| t.as_str())
		);
		Ok(())
	}

	#[test]
	fn set_empty_when_item_cannot_be_retrieved() -> Result<(), RunSystemError> {
		let mut app = setup_app([(_Key(42), _Item("my item"))]);
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
		let mut items = app.world_mut().resource_mut::<Assets<_Item>>();
		*items = Assets::default();

		app.world_mut()
			.run_system_once(_Container::set_container_panels::<_Panel, _Key>)?;

		assert_eq!(
			Some("<Empty>"),
			app.world()
				.entity(text)
				.get::<Text>()
				.map(|Text(t)| t.as_str())
		);
		Ok(())
	}

	#[test]
	fn still_set_state_when_no_children() -> Result<(), RunSystemError> {
		let mut app = setup_app([]);
		app.world_mut().spawn((
			KeyedPanel(_Key(42)),
			_Panel::new().with_mock(|mock| {
				mock.expect_set().times(1).return_const(());
			}),
		));

		app.world_mut()
			.run_system_once(_Container::set_container_panels::<_Panel, _Key>)
	}

	#[test]
	fn set_when_text_not_first_child() -> Result<(), RunSystemError> {
		let mut app = setup_app([(_Key(42), _Item("my item"))]);
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

		app.world_mut()
			.run_system_once(_Container::set_container_panels::<_Panel, _Key>)?;

		assert_eq!(
			Some("my item"),
			app.world()
				.entity(text)
				.get::<Text>()
				.map(|Text(t)| t.as_str())
		);
		Ok(())
	}
}
