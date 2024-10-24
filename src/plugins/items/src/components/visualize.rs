use super::visualizer::Visualizer;
use crate::{
	item::Item,
	traits::{key_string::KeyString, uses_view::UsesView, view_component::ViewComponent},
};
use bevy::prelude::*;
use common::{
	errors::{Error, Level},
	traits::{accessors::get::Getter, try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom},
};
use std::{collections::HashMap, marker::PhantomData};

#[derive(Component, Debug, PartialEq)]
pub struct VisualizeCommands<TView>
where
	TView: ViewComponent,
{
	phantom_data: PhantomData<TView>,
	commands: HashMap<&'static str, TView::TViewComponent>,
}

impl<TView> Default for VisualizeCommands<TView>
where
	TView: ViewComponent,
{
	fn default() -> Self {
		Self {
			phantom_data: PhantomData,
			commands: default(),
		}
	}
}

impl<TView> VisualizeCommands<TView>
where
	TView: ViewComponent,
	TView::TViewComponent: Default,
{
	pub fn with_item<TKey, TContent>(mut self, key: &TKey, item: Option<&Item<TContent>>) -> Self
	where
		TView: KeyString<TKey>,
		TContent: UsesView<TView> + Getter<TView::TViewComponent>,
	{
		let model = Self::get_model(item);
		self.commands.insert(TView::key_string(key), model);
		self
	}

	fn get_model<TContent>(item: Option<&Item<TContent>>) -> TView::TViewComponent
	where
		TContent: UsesView<TView> + Getter<TView::TViewComponent>,
	{
		let Some(item) = item else {
			return default();
		};

		if !item.content.uses_view() {
			return default();
		}

		item.content.get()
	}

	pub(crate) fn apply(
		commands: Commands,
		visualizers: Query<(Entity, &Visualizer<TView>, &VisualizeCommands<TView>)>,
	) -> Vec<Result<(), Error>>
	where
		TView: Sync + Send + 'static,
		TView::TViewComponent: Component + Clone + Sync + Send + 'static,
	{
		visualize_system(commands, visualizers)
	}
}

fn visualize_system<TCommands, TView>(
	mut commands: TCommands,
	visualizers: Query<(Entity, &Visualizer<TView>, &VisualizeCommands<TView>)>,
) -> Vec<Result<(), Error>>
where
	TCommands: TryInsertOn + TryRemoveFrom,
	TView: ViewComponent + Send + Sync + 'static,
	TView::TViewComponent: Component + Clone + Send + Sync + 'static,
{
	let mut errors = vec![];

	for (entity, visualizer, visualize) in &visualizers {
		commands.try_remove_from::<VisualizeCommands<TView>>(entity);

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

fn apply<TCommands, TView, TComponent>(
	commands: &mut TCommands,
	visualizer: &Visualizer<TView>,
	key: &'static str,
	model: &TComponent,
) -> Result<(), Error>
where
	TCommands: TryInsertOn,
	TComponent: Component + Clone,
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

	impl KeyString<_Key> for _View {
		fn key_string(key: &_Key) -> &'static str {
			match key {
				_Key::A => "a",
				_Key::B => "b",
			}
		}
	}

	impl ViewComponent for _View {
		type TViewComponent = _ViewComponent;
	}

	#[derive(Default)]
	struct _Content {
		uses_view: bool,
		model: _ViewComponent,
	}

	impl UsesView<_View> for _Content {
		fn uses_view(&self) -> bool {
			self.uses_view
		}
	}

	impl Getter<_ViewComponent> for _Content {
		fn get(&self) -> _ViewComponent {
			self.model
		}
	}

	#[test]
	fn add_item() {
		let item = Item {
			content: _Content {
				uses_view: true,
				model: _ViewComponent::Value("my model"),
			},
			..default()
		};
		let visualize = VisualizeCommands::<_View>::default().with_item(&_Key::A, Some(&item));

		assert_eq!(
			VisualizeCommands::<_View> {
				phantom_data: PhantomData,
				commands: HashMap::from([("a", _ViewComponent::Value("my model"))]),
			},
			visualize,
		)
	}

	#[test]
	fn add_none_item() {
		let visualize = VisualizeCommands::<_View>::default()
			.with_item(&_Key::A, None as Option<&Item<_Content>>);

		assert_eq!(
			VisualizeCommands::<_View> {
				phantom_data: PhantomData,
				commands: HashMap::from([("a", _ViewComponent::Default)]),
			},
			visualize,
		)
	}

	#[test]
	fn add_multiple_items() {
		let item_a = Item {
			content: _Content {
				uses_view: true,
				model: _ViewComponent::Value("my model a"),
			},
			..default()
		};
		let item_b = Item {
			content: _Content {
				uses_view: true,
				model: _ViewComponent::Value("my model b"),
			},
			..default()
		};
		let visualize = VisualizeCommands::<_View>::default()
			.with_item(&_Key::A, Some(&item_a))
			.with_item(&_Key::B, Some(&item_b));

		assert_eq!(
			VisualizeCommands::<_View> {
				phantom_data: PhantomData,
				commands: HashMap::from([
					("a", _ViewComponent::Value("my model a")),
					("b", _ViewComponent::Value("my model b"))
				]),
			},
			visualize,
		)
	}

	#[test]
	fn add_item_with_mode_none_when_not_using_view() {
		let item = Item {
			content: _Content {
				uses_view: false,
				model: _ViewComponent::Value("my model"),
			},
			..default()
		};
		let visualize = VisualizeCommands::<_View>::default().with_item(&_Key::A, Some(&item));

		assert_eq!(
			VisualizeCommands::<_View> {
				phantom_data: PhantomData,
				commands: HashMap::from([("a", _ViewComponent::Default)]),
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
			Visualizer::<_View>::new([(Name::from("a"), Entity::from_raw(42))]),
			VisualizeCommands::<_View> {
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
			mock.expect_try_remove_from::<VisualizeCommands<_View>>()
				.return_const(());
		});

		app.world_mut()
			.run_system_once_with(commands, visualize_system::<In<Mock_Commands>, _View>);
	}

	#[test]
	fn visualize_none() {
		let mut app = setup();
		app.world_mut().spawn((
			Visualizer::<_View>::new([(Name::from("a"), Entity::from_raw(42))]),
			VisualizeCommands::<_View> {
				commands: HashMap::from([("a", _ViewComponent::Default)]),
				..default()
			},
		));

		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_insert_on()
				.times(1)
				.with(eq(Entity::from_raw(42)), eq(_ViewComponent::Default))
				.return_const(());
			mock.expect_try_remove_from::<VisualizeCommands<_View>>()
				.return_const(());
		});

		app.world_mut()
			.run_system_once_with(commands, visualize_system::<In<Mock_Commands>, _View>);
	}

	#[test]
	fn remove_visualize_component() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Visualizer::<_View>::new([(Name::from("a"), Entity::from_raw(42))]),
				VisualizeCommands::<_View> {
					commands: HashMap::from([("a", _ViewComponent::Value("my model"))]),
					..default()
				},
			))
			.id();

		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_insert_on::<_ViewComponent>()
				.return_const(());
			mock.expect_try_remove_from::<VisualizeCommands<_View>>()
				.times(1)
				.with(eq(entity))
				.return_const(());
		});

		app.world_mut()
			.run_system_once_with(commands, visualize_system::<In<Mock_Commands>, _View>);
	}

	#[test]
	fn remove_visualize_component_even_when_commands_empty() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Visualizer::<_View>::new([(Name::from("a"), Entity::from_raw(42))]),
				VisualizeCommands::<_View> {
					commands: HashMap::new(),
					..default()
				},
			))
			.id();

		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_insert_on::<_ViewComponent>()
				.return_const(());
			mock.expect_try_remove_from::<VisualizeCommands<_View>>()
				.times(1)
				.with(eq(entity))
				.return_const(());
		});

		app.world_mut()
			.run_system_once_with(commands, visualize_system::<In<Mock_Commands>, _View>);
	}

	#[test]
	fn return_error_when_key_entity_not_found() {
		let mut app = setup();
		app.world_mut().spawn((
			Visualizer::<_View>::new([(Name::from("a"), Entity::from_raw(42))]),
			VisualizeCommands::<_View> {
				commands: HashMap::from([("other key", _ViewComponent::Value("my model"))]),
				..default()
			},
		));
		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_insert_on::<_ViewComponent>()
				.never()
				.return_const(());
			mock.expect_try_remove_from::<VisualizeCommands<_View>>()
				.return_const(());
		});

		let results = app
			.world_mut()
			.run_system_once_with(commands, visualize_system::<In<Mock_Commands>, _View>);

		assert_eq!(vec![Err(entity_not_found_error("other key"))], results);
	}
}
