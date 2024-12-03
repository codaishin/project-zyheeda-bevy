use std::fmt::Debug;

use crate::{components::KeyedPanel, tools::PanelState};
use bevy::{hierarchy::Parent, prelude::*};
use common::traits::accessors::{get::GetRef, set::Setter};
use skills::item::Item;

pub fn panel_container_states<TPanel, TKey, TContainer>(
	containers: Query<&TContainer>,
	mut texts: Query<(&Parent, &mut Text)>,
	mut panels: Query<(Entity, &KeyedPanel<TKey>, &mut TPanel)>,
	items: Res<Assets<Item>>,
) where
	TPanel: Component + Setter<PanelState>,
	TKey: Debug + Copy + Send + Sync + 'static,
	TContainer: Component + GetRef<TKey, Handle<Item>>,
{
	let container = containers.single();

	for (entity, KeyedPanel(key), mut panel) in &mut panels {
		let (state, label) = match get_item(container, key, &items) {
			Some(item) => (PanelState::Filled, item.name.clone()),
			None => (PanelState::Empty, "<Empty>".to_owned()),
		};
		panel.set(state);
		set_label(&mut texts, entity, label);
	}
}

fn get_item<'a, TContainer, TKey>(
	container: &'a TContainer,
	key: &TKey,
	items: &'a Assets<Item>,
) -> Option<&'a Item>
where
	TContainer: GetRef<TKey, Handle<Item>>,
{
	container.get(key).and_then(|item| items.get(item))
}

fn set_label(texts: &mut Query<(&Parent, &mut Text)>, entity: Entity, label: String) {
	let Some((.., mut text)) = texts.iter_mut().find(|(p, ..)| p.get() == entity) else {
		return;
	};

	if text.sections.is_empty() {
		text.sections.push(label.into());
	} else {
		text.sections[0].value = label;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::RunSystemOnce;
	use common::traits::nested_mock::NestedMocks;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashMap;

	#[derive(Component)]
	struct _Container(HashMap<usize, Handle<Item>>);

	impl GetRef<usize, Handle<Item>> for _Container {
		fn get(&self, key: &usize) -> Option<&Handle<Item>> {
			self.0.get(key)
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

	fn setup_container<const N: usize>(
		items: [(usize, Item); N],
	) -> (_Container, Assets<Item>) {
		let mut item_assets = Assets::default();
		let mut container = HashMap::default();

		for (key, item) in items {
			let item = item_assets.add(item);
			container.insert(key, item);
		}

		(_Container(container), item_assets)
	}

	fn setup_app<const N: usize>(items: [(usize, Item); N]) -> App {
		let (container, items) = setup_container(items);
		let mut app = App::new();
		app.insert_resource(items);
		app.world_mut().spawn(container);

		app
	}

	#[test]
	fn set_empty() {
		let mut app = setup_app([]);
		let panel = app
			.world_mut()
			.spawn((
				KeyedPanel(42_usize),
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
			.spawn(TextBundle::from_section("", default()))
			.set_parent(panel)
			.id();

		app.world_mut()
			.run_system_once(panel_container_states::<_Panel, usize, _Container>);

		let text = app.world().entity(text).get::<Text>().unwrap();
		assert_eq!("<Empty>", text.sections[0].value);
	}

	#[test]
	fn set_filled() {
		let mut app = setup_app([(42, Item::named("my item"))]);
		let panel = app
			.world_mut()
			.spawn((
				KeyedPanel(42_usize),
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
			.spawn(TextBundle::from_section("", default()))
			.set_parent(panel)
			.id();

		app.world_mut()
			.run_system_once(panel_container_states::<_Panel, usize, _Container>);

		let text = app.world().entity(text).get::<Text>().unwrap();
		assert_eq!("my item", text.sections[0].value);
	}

	#[test]
	fn set_empty_when_item_cannot_be_retrieved() {
		let mut app = setup_app([(42, Item::named("my item"))]);
		let panel = app
			.world_mut()
			.spawn((
				KeyedPanel(42_usize),
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
			.spawn(TextBundle::from_section("", default()))
			.set_parent(panel)
			.id();
		let mut items = app.world_mut().resource_mut::<Assets<Item>>();
		*items = Assets::default();

		app.world_mut()
			.run_system_once(panel_container_states::<_Panel, usize, _Container>);

		let text = app.world().entity(text).get::<Text>().unwrap();
		assert_eq!("<Empty>", text.sections[0].value.as_str());
	}

	#[test]
	fn still_set_state_when_no_children() {
		let mut app = setup_app([]);
		app.world_mut().spawn((
			KeyedPanel(42_usize),
			_Panel::new().with_mock(|mock| {
				mock.expect_set().times(1).return_const(());
			}),
		));

		app.world_mut()
			.run_system_once(panel_container_states::<_Panel, usize, _Container>);
	}

	#[test]
	fn set_when_text_not_first_child() {
		let mut app = setup_app([(42, Item::named("my item"))]);
		let panel = app
			.world_mut()
			.spawn((
				KeyedPanel(42_usize),
				_Panel::new().with_mock(|mock| {
					mock.expect_set()
						.times(1)
						.with(eq(PanelState::Filled))
						.return_const(());
				}),
			))
			.id();
		app.world_mut().spawn(()).set_parent(panel);
		let text = app
			.world_mut()
			.spawn(TextBundle::from_section("", default()))
			.set_parent(panel)
			.id();

		app.world_mut()
			.run_system_once(panel_container_states::<_Panel, usize, _Container>);

		let text = app.world().entity(text).get::<Text>().unwrap();
		assert_eq!("my item", text.sections[0].value);
	}

	#[test]
	fn add_section_when_text_has_no_sections() {
		let mut app = setup_app([(42, Item::named("my item"))]);
		let panel = app
			.world_mut()
			.spawn((
				KeyedPanel(42_usize),
				_Panel::new().with_mock(|mock| {
					mock.expect_set().return_const(());
				}),
			))
			.id();
		app.world_mut().spawn(()).set_parent(panel);
		let text = app
			.world_mut()
			.spawn(Text::default())
			.set_parent(panel)
			.id();

		app.world_mut()
			.run_system_once(panel_container_states::<_Panel, usize, _Container>);

		let text = app.world().entity(text).get::<Text>().unwrap();
		assert_eq!("my item", text.sections[0].value);
	}
}
