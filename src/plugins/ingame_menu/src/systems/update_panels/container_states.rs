use crate::{components::KeyedPanel, tools::PanelState, traits::set::Set};
use bevy::{
	ecs::{component::Component, system::Query},
	hierarchy::Parent,
	prelude::Entity,
	text::Text,
};
use common::traits::get::Get;
use skills::items::Item;

pub fn panel_container_states<
	TPanel: Component + Set<(), PanelState>,
	TKey: Copy + Send + Sync + 'static,
	TContainer: Component + Get<TKey, Item>,
>(
	containers: Query<&TContainer>,
	mut texts: Query<(&Parent, &mut Text)>,
	mut panels: Query<(Entity, &KeyedPanel<TKey>, &mut TPanel)>,
) {
	let container = containers.single();

	for (entity, keyed_panel, mut panel) in &mut panels {
		let (state, label) = match container.get(&keyed_panel.0) {
			Some(item) => (PanelState::Filled, item.name),
			None => (PanelState::Empty, "<Empty>"),
		};
		panel.set((), state);
		set_label(&mut texts, entity, label);
	}
}

fn set_label(texts: &mut Query<(&Parent, &mut Text)>, entity: Entity, label: &str) {
	let Some((.., mut text)) = texts.iter_mut().find(|(p, ..)| p.get() == entity) else {
		return;
	};

	if text.sections.is_empty() {
		text.sections.push(label.into());
	} else {
		text.sections[0].value = label.into();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		hierarchy::BuildWorldChildren,
		prelude::default,
		ui::node_bundles::TextBundle,
		utils::HashMap,
	};
	use common::traits::nested_mock::NestedMocks;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Component)]
	struct _Container(HashMap<usize, Item>);

	impl _Container {
		pub fn new<const N: usize>(items: [(usize, Item); N]) -> Self {
			Self(HashMap::from(items))
		}
	}

	impl Get<usize, Item> for _Container {
		fn get(&self, key: &usize) -> Option<&Item> {
			self.0.get(key)
		}
	}

	#[derive(Component, NestedMocks)]
	struct _Panel {
		mock: Mock_Panel,
	}

	#[automock]
	impl Set<(), PanelState> for _Panel {
		fn set(&mut self, key: (), value: PanelState) {
			self.mock.set(key, value)
		}
	}

	#[test]
	fn set_empty() {
		let mut app = App::new();
		app.add_systems(Update, panel_container_states::<_Panel, usize, _Container>);
		app.world_mut().spawn(_Container::new([]));
		let panel = app
			.world_mut()
			.spawn((
				KeyedPanel(42_usize),
				_Panel::new().with_mock(|mock| {
					mock.expect_set()
						.times(1)
						.with(eq(()), eq(PanelState::Empty))
						.return_const(());
				}),
			))
			.id();
		let text = app
			.world_mut()
			.spawn(TextBundle::from_section("", default()))
			.set_parent(panel)
			.id();

		app.update();

		let text = app.world().entity(text).get::<Text>().unwrap();
		assert_eq!("<Empty>", text.sections[0].value);
	}

	#[test]
	fn set_filled() {
		let mut app = App::new();
		app.add_systems(Update, panel_container_states::<_Panel, usize, _Container>);
		app.world_mut().spawn(_Container::new([(
			42,
			Item {
				name: "my item",
				..default()
			},
		)]));
		let panel = app
			.world_mut()
			.spawn((
				KeyedPanel(42_usize),
				_Panel::new().with_mock(|mock| {
					mock.expect_set()
						.times(1)
						.with(eq(()), eq(PanelState::Filled))
						.return_const(());
				}),
			))
			.id();
		let text = app
			.world_mut()
			.spawn(TextBundle::from_section("", default()))
			.set_parent(panel)
			.id();

		app.update();

		let text = app.world().entity(text).get::<Text>().unwrap();
		assert_eq!("my item", text.sections[0].value);
	}

	#[test]
	fn still_set_state_when_no_children() {
		let mut app = App::new();
		app.add_systems(Update, panel_container_states::<_Panel, usize, _Container>);
		app.world_mut().spawn(_Container::new([]));
		app.world_mut().spawn((
			KeyedPanel(42_usize),
			_Panel::new().with_mock(|mock| {
				mock.expect_set().times(1).return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn set_when_text_not_first_child() {
		let mut app = App::new();
		app.add_systems(Update, panel_container_states::<_Panel, usize, _Container>);
		app.world_mut().spawn(_Container::new([(
			42,
			Item {
				name: "my item",
				..default()
			},
		)]));
		let panel = app
			.world_mut()
			.spawn((
				KeyedPanel(42_usize),
				_Panel::new().with_mock(|mock| {
					mock.expect_set()
						.times(1)
						.with(eq(()), eq(PanelState::Filled))
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

		app.update();

		let text = app.world().entity(text).get::<Text>().unwrap();
		assert_eq!("my item", text.sections[0].value);
	}

	#[test]
	fn add_section_when_text_has_no_sections() {
		let mut app = App::new();
		app.add_systems(Update, panel_container_states::<_Panel, usize, _Container>);
		app.world_mut().spawn(_Container::new([(
			42,
			Item {
				name: "my item",
				..default()
			},
		)]));
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

		app.update();

		let text = app.world().entity(text).get::<Text>().unwrap();
		assert_eq!("my item", text.sections[0].value);
	}
}
