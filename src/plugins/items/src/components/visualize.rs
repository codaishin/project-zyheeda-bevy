use super::visualizer::Visualizer;
use crate::{
	item::Item,
	traits::{
		item_type::AssociatedItemType,
		key_string::KeyString,
		uses_visualizer::UsesVisualizer,
	},
};
use bevy::prelude::*;
use common::{
	errors::{Error, Level},
	resources::Models,
	traits::{try_complex_insert::TryComplexInsert, try_remove_from::TryRemoveFrom},
};
use std::{collections::HashMap, marker::PhantomData};

#[derive(Component, Debug, PartialEq)]
pub struct Visualize<TVisualizer> {
	phantom_data: PhantomData<TVisualizer>,
	items: HashMap<&'static str, Option<&'static str>>,
}

impl<TVisualizer> Default for Visualize<TVisualizer> {
	fn default() -> Self {
		Self {
			phantom_data: PhantomData,
			items: default(),
		}
	}
}

impl<TVisualizer> Visualize<TVisualizer> {
	pub fn with_item<TKey, TContent>(mut self, key: &TKey, item: Option<&Item<TContent>>) -> Self
	where
		TVisualizer: KeyString<TKey>,
		TContent: AssociatedItemType,
		TContent::TItemType: UsesVisualizer<TVisualizer>,
	{
		let model = Self::get_model(item);
		self.items.insert(TVisualizer::key_string(key), model);
		self
	}

	fn get_model<TContent>(item: Option<&Item<TContent>>) -> Option<&'static str>
	where
		TContent: AssociatedItemType,
		TContent::TItemType: UsesVisualizer<TVisualizer>,
	{
		let item = item?;

		if !item.item_type.uses_visualization() {
			return None;
		}

		item.model
	}

	pub(crate) fn system(
		commands: Commands,
		visualizers: Query<(Entity, &Visualizer<TVisualizer>, &Visualize<TVisualizer>)>,
		models: Res<Models>,
	) -> Vec<Result<(), Error>>
	where
		TVisualizer: Sync + Send + 'static,
	{
		visualize_system(commands, visualizers, models)
	}
}

fn visualize_system<TCommands, TVisualizer>(
	mut commands: TCommands,
	visualizers: Query<(Entity, &Visualizer<TVisualizer>, &Visualize<TVisualizer>)>,
	models: Res<Models>,
) -> Vec<Result<(), Error>>
where
	TCommands: TryComplexInsert<Option<Handle<Scene>>> + TryRemoveFrom,
	TVisualizer: Send + Sync + 'static,
{
	let mut errors = vec![];

	for (entity, visualizer, visualize) in &visualizers {
		for (key, model) in &visualize.items {
			let result = apply(&mut commands, &models, visualizer, key, model);
			commands.try_remove_from::<Visualize<TVisualizer>>(entity);

			let Err(error) = result else {
				continue;
			};
			errors.push(Err(error));
		}
	}

	errors
}

fn apply<TCommands, TVisualizer>(
	commands: &mut TCommands,
	models: &Res<'_, Models>,
	visualizer: &Visualizer<TVisualizer>,
	key: &'static str,
	model: &Option<&'static str>,
) -> Result<(), Error>
where
	TCommands: TryComplexInsert<Option<Handle<Scene>>>,
{
	let Some(entity) = visualizer.entities.get(&Name::from(key)) else {
		return Err(entity_not_found_error(key));
	};

	match model {
		None => {
			commands.try_complex_insert(*entity, None);
			Ok(())
		}
		Some(model) => {
			let Some(model) = models.0.get(model) else {
				return Err(model_not_found_error(model));
			};
			commands.try_complex_insert(*entity, Some(model.clone()));
			Ok(())
		}
	}
}

fn entity_not_found_error(key: &'static str) -> Error {
	Error {
		msg: format!("no entity found for {key}"),
		lvl: Level::Error,
	}
}

fn model_not_found_error(model: &'static str) -> Error {
	Error {
		msg: format!("model for path {model} not found"),
		lvl: Level::Error,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::visualizer::Visualizer;
	use bevy::ecs::system::RunSystemOnce;
	use common::{
		resources::Models,
		simple_init,
		test_tools::utils::new_handle,
		traits::{mock::Mock, try_complex_insert::TryComplexInsert},
	};
	use mockall::{mock, predicate::eq};

	enum _Key {
		A,
		B,
	}

	#[derive(Debug, PartialEq)]
	struct _Visualizer;

	impl KeyString<_Key> for _Visualizer {
		fn key_string(key: &_Key) -> &'static str {
			match key {
				_Key::A => "a",
				_Key::B => "b",
			}
		}
	}

	struct _Content;

	impl AssociatedItemType for _Content {
		type TItemType = _ContentItemType;
	}

	#[derive(Default)]
	struct _ContentItemType {
		uses_visualizer: bool,
	}

	impl UsesVisualizer<_Visualizer> for _ContentItemType {
		fn uses_visualization(&self) -> bool {
			self.uses_visualizer
		}
	}

	#[test]
	fn add_item() {
		let item = Item::<_Content> {
			model: Some("my model"),
			item_type: _ContentItemType {
				uses_visualizer: true,
			},
			..default()
		};
		let visualize = Visualize::<_Visualizer>::default().with_item(&_Key::A, Some(&item));

		assert_eq!(
			Visualize::<_Visualizer> {
				phantom_data: PhantomData,
				items: HashMap::from([("a", Some("my model"))]),
			},
			visualize,
		)
	}

	#[test]
	fn add_none_item() {
		let visualize = Visualize::<_Visualizer>::default()
			.with_item(&_Key::A, None as Option<&Item<_Content>>);

		assert_eq!(
			Visualize::<_Visualizer> {
				phantom_data: PhantomData,
				items: HashMap::from([("a", None)]),
			},
			visualize,
		)
	}

	#[test]
	fn add_multiple_items() {
		let item_a = Item::<_Content> {
			model: Some("my model a"),
			item_type: _ContentItemType {
				uses_visualizer: true,
			},
			..default()
		};
		let item_b = Item::<_Content> {
			model: Some("my model b"),
			item_type: _ContentItemType {
				uses_visualizer: true,
			},
			..default()
		};
		let visualize = Visualize::<_Visualizer>::default()
			.with_item(&_Key::A, Some(&item_a))
			.with_item(&_Key::B, Some(&item_b));

		assert_eq!(
			Visualize::<_Visualizer> {
				phantom_data: PhantomData,
				items: HashMap::from([("a", Some("my model a")), ("b", Some("my model b"))]),
			},
			visualize,
		)
	}

	#[test]
	fn add_item_with_mode_none_when_not_using_visualizer() {
		let item = Item::<_Content> {
			model: Some("my model"),
			item_type: _ContentItemType {
				uses_visualizer: false,
			},
			..default()
		};
		let visualize = Visualize::<_Visualizer>::default().with_item(&_Key::A, Some(&item));

		assert_eq!(
			Visualize::<_Visualizer> {
				phantom_data: PhantomData,
				items: HashMap::from([("a", None as Option<&str>)]),
			},
			visualize,
		)
	}

	mock! {
		_Commands {}
		impl TryComplexInsert<Option<Handle<Scene>>> for _Commands {
			fn try_complex_insert(&mut self, entity: Entity, value: Option<Handle<Scene>>);
		}
		impl TryRemoveFrom for _Commands {
				fn try_remove_from<TBundle: Bundle>(&mut self, entity: Entity);
		}
	}

	simple_init!(Mock_Commands);

	fn setup<const N: usize>(models: [(&'static str, Handle<Scene>); N]) -> App {
		let mut app = App::new();
		app.insert_resource(Models(HashMap::from(models)));
		app
	}

	#[test]
	fn visualize_scene() {
		let handle = new_handle();
		let mut app = setup([("my model", handle.clone())]);
		app.world_mut().spawn((
			Visualizer::<_Visualizer>::new([(Name::from("a"), Entity::from_raw(42))]),
			Visualize::<_Visualizer> {
				items: HashMap::from([("a", Some("my model"))]),
				..default()
			},
		));

		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_complex_insert()
				.times(1)
				.with(eq(Entity::from_raw(42)), eq(Some(handle.clone())))
				.return_const(());
			mock.expect_try_remove_from::<Visualize<_Visualizer>>()
				.return_const(());
		});

		app.world_mut()
			.run_system_once_with(commands, visualize_system::<In<Mock_Commands>, _Visualizer>);
	}

	#[test]
	fn visualize_none() {
		let mut app = setup([]);
		app.world_mut().spawn((
			Visualizer::<_Visualizer>::new([(Name::from("a"), Entity::from_raw(42))]),
			Visualize::<_Visualizer> {
				items: HashMap::from([("a", None)]),
				..default()
			},
		));

		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_complex_insert()
				.times(1)
				.with(eq(Entity::from_raw(42)), eq(None))
				.return_const(());
			mock.expect_try_remove_from::<Visualize<_Visualizer>>()
				.return_const(());
		});

		app.world_mut()
			.run_system_once_with(commands, visualize_system::<In<Mock_Commands>, _Visualizer>);
	}

	#[test]
	fn remove_visualize_component() {
		let handle = new_handle();
		let mut app = setup([("my model", handle.clone())]);
		let entity = app
			.world_mut()
			.spawn((
				Visualizer::<_Visualizer>::new([(Name::from("a"), Entity::from_raw(42))]),
				Visualize::<_Visualizer> {
					items: HashMap::from([("a", Some("my model"))]),
					..default()
				},
			))
			.id();

		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_complex_insert().return_const(());
			mock.expect_try_remove_from::<Visualize<_Visualizer>>()
				.times(1)
				.with(eq(entity))
				.return_const(());
		});

		app.world_mut()
			.run_system_once_with(commands, visualize_system::<In<Mock_Commands>, _Visualizer>);
	}

	#[test]
	fn return_error_when_model_not_found() {
		let handle = new_handle();
		let mut app = setup([("my model", handle.clone())]);
		app.world_mut().spawn((
			Visualizer::<_Visualizer>::new([(Name::from("a"), Entity::from_raw(42))]),
			Visualize::<_Visualizer> {
				items: HashMap::from([("a", Some("other model"))]),
				..default()
			},
		));
		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_complex_insert().never().return_const(());
			mock.expect_try_remove_from::<Visualize<_Visualizer>>()
				.return_const(());
		});

		let results = app
			.world_mut()
			.run_system_once_with(commands, visualize_system::<In<Mock_Commands>, _Visualizer>);

		assert_eq!(vec![Err(model_not_found_error("other model"))], results);
	}

	#[test]
	fn return_error_when_key_entity_not_found() {
		let handle = new_handle();
		let mut app = setup([("my model", handle.clone())]);
		app.world_mut().spawn((
			Visualizer::<_Visualizer>::new([(Name::from("a"), Entity::from_raw(42))]),
			Visualize::<_Visualizer> {
				items: HashMap::from([("other key", Some("my model"))]),
				..default()
			},
		));
		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_complex_insert().never().return_const(());
			mock.expect_try_remove_from::<Visualize<_Visualizer>>()
				.return_const(());
		});

		let results = app
			.world_mut()
			.run_system_once_with(commands, visualize_system::<In<Mock_Commands>, _Visualizer>);

		assert_eq!(vec![Err(entity_not_found_error("other key"))], results);
	}
}
