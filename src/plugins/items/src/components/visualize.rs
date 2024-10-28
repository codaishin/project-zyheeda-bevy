use super::visualizer::Visualizer;
use crate::{
	item::Item,
	traits::{get_view_data::GetViewData, view::ItemView},
};
use bevy::prelude::*;
use common::{
	errors::{Error, Level},
	traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom},
};
use std::{collections::HashMap, marker::PhantomData};

#[derive(Component, Debug, PartialEq)]
pub struct VisualizeCommands<TView, TKey>
where
	TView: ItemView<TKey>,
{
	phantom_data: PhantomData<(TView, TKey)>,
	commands: HashMap<&'static str, TView::TViewComponents>,
}

impl<TView, TKey> Default for VisualizeCommands<TView, TKey>
where
	TView: ItemView<TKey>,
{
	fn default() -> Self {
		Self {
			phantom_data: PhantomData,
			commands: default(),
		}
	}
}

impl<TView, TKey> VisualizeCommands<TView, TKey>
where
	TView: ItemView<TKey>,
{
	pub fn with_item<TContent>(mut self, key: &TKey, item: Option<&Item<TContent>>) -> Self
	where
		TView: ItemView<TKey>,
		TContent: GetViewData<TView, TKey>,
	{
		let components = Self::view_components(item);
		let view_slot = TView::view_entity_name(key);
		self.commands.insert(view_slot, components);
		self
	}

	fn view_components<TContent>(item: Option<&Item<TContent>>) -> TView::TViewComponents
	where
		TContent: GetViewData<TView, TKey>,
	{
		let Some(item) = item else {
			return default();
		};

		item.content.get_view_data()
	}

	pub(crate) fn apply(
		commands: Commands,
		visualizers: Query<VisualizerComponents<TView, TKey>>,
	) -> Vec<Result<(), Error>>
	where
		TView: Sync + Send + 'static,
		TKey: Sync + Send + 'static,
	{
		visualize_system(commands, visualizers)
	}
}

type VisualizerComponents<'a, TView, TKey> = (
	Entity,
	&'a Visualizer<TView, TKey>,
	&'a VisualizeCommands<TView, TKey>,
);

fn visualize_system<TCommands, TView, TKey>(
	mut commands: TCommands,
	visualizers: Query<VisualizerComponents<TView, TKey>>,
) -> Vec<Result<(), Error>>
where
	TCommands: TryInsertOn + TryRemoveFrom,
	TView: ItemView<TKey> + Send + Sync + 'static,
	TKey: Sync + Send + 'static,
{
	let mut errors = vec![];

	for (entity, visualizer, visualize) in &visualizers {
		commands.try_remove_from::<VisualizeCommands<TView, TKey>>(entity);

		for (key, model) in &visualize.commands {
			let result = apply(&mut commands, visualizer, key, model);

			let Err(error) = result else {
				continue;
			};
			errors.push(Err(error));
		}
	}

	errors
}

fn apply<TCommands, TView, TKey, TComponent>(
	commands: &mut TCommands,
	visualizer: &Visualizer<TView, TKey>,
	key: &'static str,
	model: &TComponent,
) -> Result<(), Error>
where
	TCommands: TryInsertOn,
	TComponent: Bundle + Clone,
{
	let entity = visualizer
		.entities
		.get(&Name::from(key))
		.ok_or(entity_not_found_error(key))?;

	commands.try_insert_on(*entity, model.clone());
	Ok(())
}

fn entity_not_found_error(key: &'static str) -> Error {
	Error {
		msg: format!("no entity found for {key}"),
		lvl: Level::Error,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::visualizer::Visualizer;
	use bevy::ecs::system::RunSystemOnce;
	use common::{simple_init, traits::mock::Mock};
	use mockall::{mock, predicate::eq};

	#[derive(Debug, PartialEq)]
	enum _Key {
		A,
		B,
	}

	#[derive(Debug, PartialEq)]
	struct _View;

	#[derive(Component, Debug, PartialEq, Default, Clone, Copy)]
	enum _ViewComponent {
		#[default]
		Default,
		Value(&'static str),
	}

	impl ItemView<_Key> for _View {
		type TFilter = ();
		type TViewComponents = _ViewComponent;

		fn view_entity_name(key: &_Key) -> &'static str {
			match key {
				_Key::A => "a",
				_Key::B => "b",
			}
		}
	}

	#[derive(Default)]
	struct _Content(_ViewComponent);

	impl GetViewData<_View, _Key> for _Content {
		fn get_view_data(&self) -> <_View as ItemView<_Key>>::TViewComponents {
			self.0
		}
	}

	#[test]
	fn add_item() {
		let item = Item {
			content: _Content(_ViewComponent::Value("my model")),
			..default()
		};
		let visualize =
			VisualizeCommands::<_View, _Key>::default().with_item(&_Key::A, Some(&item));

		assert_eq!(
			VisualizeCommands::<_View, _Key> {
				phantom_data: PhantomData,
				commands: HashMap::from([("a", _ViewComponent::Value("my model"))]),
			},
			visualize,
		)
	}

	#[test]
	fn add_none_item() {
		let visualize = VisualizeCommands::<_View, _Key>::default()
			.with_item(&_Key::A, None as Option<&Item<_Content>>);

		assert_eq!(
			VisualizeCommands::<_View, _Key> {
				phantom_data: PhantomData,
				commands: HashMap::from([("a", _ViewComponent::Default)]),
			},
			visualize,
		)
	}

	#[test]
	fn add_multiple_items() {
		let item_a = Item {
			content: _Content(_ViewComponent::Value("my model a")),
			..default()
		};
		let item_b = Item {
			content: _Content(_ViewComponent::Value("my model b")),
			..default()
		};
		let visualize = VisualizeCommands::<_View, _Key>::default()
			.with_item(&_Key::A, Some(&item_a))
			.with_item(&_Key::B, Some(&item_b));

		assert_eq!(
			VisualizeCommands::<_View, _Key> {
				phantom_data: PhantomData,
				commands: HashMap::from([
					("a", _ViewComponent::Value("my model a")),
					("b", _ViewComponent::Value("my model b"))
				]),
			},
			visualize,
		)
	}

	mock! {
		_Commands {}
		impl TryInsertOn for _Commands {
			fn try_insert_on<TBundle: Bundle>(&mut self, entity: Entity, bundle: TBundle);
		}
		impl TryRemoveFrom for _Commands {
			fn try_remove_from<TBundle: Bundle>(&mut self, entity: Entity);
		}
	}

	simple_init!(Mock_Commands);

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn visualize_scene() {
		let mut app = setup();
		app.world_mut().spawn((
			Visualizer::<_View, _Key>::new([(Name::from("a"), Entity::from_raw(42))]),
			VisualizeCommands::<_View, _Key> {
				commands: HashMap::from([("a", _ViewComponent::Value("my model"))]),
				..default()
			},
		));

		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_insert_on()
				.times(1)
				.with(
					eq(Entity::from_raw(42)),
					eq(_ViewComponent::Value("my model")),
				)
				.return_const(());
			mock.expect_try_remove_from::<VisualizeCommands<_View, _Key>>()
				.return_const(());
		});

		app.world_mut()
			.run_system_once_with(commands, visualize_system::<In<Mock_Commands>, _View, _Key>);
	}

	#[test]
	fn visualize_none() {
		let mut app = setup();
		app.world_mut().spawn((
			Visualizer::<_View, _Key>::new([(Name::from("a"), Entity::from_raw(42))]),
			VisualizeCommands::<_View, _Key> {
				commands: HashMap::from([("a", _ViewComponent::Default)]),
				..default()
			},
		));

		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_insert_on()
				.times(1)
				.with(eq(Entity::from_raw(42)), eq(_ViewComponent::Default))
				.return_const(());
			mock.expect_try_remove_from::<VisualizeCommands<_View, _Key>>()
				.return_const(());
		});

		app.world_mut()
			.run_system_once_with(commands, visualize_system::<In<Mock_Commands>, _View, _Key>);
	}

	#[test]
	fn remove_visualize_component() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Visualizer::<_View, _Key>::new([(Name::from("a"), Entity::from_raw(42))]),
				VisualizeCommands::<_View, _Key> {
					commands: HashMap::from([("a", _ViewComponent::Value("my model"))]),
					..default()
				},
			))
			.id();

		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_insert_on::<_ViewComponent>()
				.return_const(());
			mock.expect_try_remove_from::<VisualizeCommands<_View, _Key>>()
				.times(1)
				.with(eq(entity))
				.return_const(());
		});

		app.world_mut()
			.run_system_once_with(commands, visualize_system::<In<Mock_Commands>, _View, _Key>);
	}

	#[test]
	fn remove_visualize_component_even_when_commands_empty() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Visualizer::<_View, _Key>::new([(Name::from("a"), Entity::from_raw(42))]),
				VisualizeCommands::<_View, _Key> {
					commands: HashMap::new(),
					..default()
				},
			))
			.id();

		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_insert_on::<_ViewComponent>()
				.return_const(());
			mock.expect_try_remove_from::<VisualizeCommands<_View, _Key>>()
				.times(1)
				.with(eq(entity))
				.return_const(());
		});

		app.world_mut()
			.run_system_once_with(commands, visualize_system::<In<Mock_Commands>, _View, _Key>);
	}

	#[test]
	fn return_error_when_key_entity_not_found() {
		let mut app = setup();
		app.world_mut().spawn((
			Visualizer::<_View, _Key>::new([(Name::from("a"), Entity::from_raw(42))]),
			VisualizeCommands::<_View, _Key> {
				commands: HashMap::from([("other key", _ViewComponent::Value("my model"))]),
				..default()
			},
		));
		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_insert_on::<_ViewComponent>()
				.never()
				.return_const(());
			mock.expect_try_remove_from::<VisualizeCommands<_View, _Key>>()
				.return_const(());
		});

		let results = app
			.world_mut()
			.run_system_once_with(commands, visualize_system::<In<Mock_Commands>, _View, _Key>);

		assert_eq!(vec![Err(entity_not_found_error("other key"))], results);
	}
}
