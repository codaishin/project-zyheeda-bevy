use super::visualizer::Visualizer;
use crate::{
	item::Item,
	traits::{key_string::KeyString, uses_view::UsesView},
};
use bevy::prelude::*;
use common::{
	errors::{Error, Level},
	tools::ModelPath,
	traits::{
		accessors::get::Getter,
		load::Load,
		try_complex_insert::TryComplexInsert,
		try_remove_from::TryRemoveFrom,
	},
};
use std::{collections::HashMap, marker::PhantomData};

#[derive(Component, Debug, PartialEq)]
pub struct VisualizeCommands<TView> {
	phantom_data: PhantomData<TView>,
	commands: HashMap<&'static str, Option<ModelPath>>,
}

impl<TView> Default for VisualizeCommands<TView> {
	fn default() -> Self {
		Self {
			phantom_data: PhantomData,
			commands: default(),
		}
	}
}

impl<TView> VisualizeCommands<TView> {
	pub fn with_item<TKey, TContent>(mut self, key: &TKey, item: Option<&Item<TContent>>) -> Self
	where
		TView: KeyString<TKey>,
		TContent: UsesView<TView> + Getter<Option<ModelPath>>,
	{
		let model = Self::get_model(item);
		self.commands.insert(TView::key_string(key), model);
		self
	}

	fn get_model<TContent>(item: Option<&Item<TContent>>) -> Option<ModelPath>
	where
		TContent: UsesView<TView> + Getter<Option<ModelPath>>,
	{
		let item = item?;

		if !item.content.uses_view() {
			return None;
		}

		item.content.get()
	}

	pub(crate) fn apply(
		commands: Commands,
		visualizers: Query<(Entity, &Visualizer<TView>, &VisualizeCommands<TView>)>,
		asset_server: Res<AssetServer>,
	) -> Vec<Result<(), Error>>
	where
		TView: Sync + Send + 'static,
	{
		visualize_system(commands, visualizers, asset_server)
	}
}

fn visualize_system<TCommands, TAssetServer, TView>(
	mut commands: TCommands,
	visualizers: Query<(Entity, &Visualizer<TView>, &VisualizeCommands<TView>)>,
	asset_server: Res<TAssetServer>,
) -> Vec<Result<(), Error>>
where
	TCommands: TryComplexInsert<Option<Handle<Scene>>> + TryRemoveFrom,
	TAssetServer: Load<ModelPath, Handle<Scene>> + Resource,
	TView: Send + Sync + 'static,
{
	let mut errors = vec![];

	for (entity, visualizer, visualize) in &visualizers {
		commands.try_remove_from::<VisualizeCommands<TView>>(entity);

		for (key, model) in &visualize.commands {
			let result = apply(&mut commands, &asset_server, visualizer, key, model);

			let Err(error) = result else {
				continue;
			};
			errors.push(Err(error));
		}
	}

	errors
}

fn apply<TCommands, TAssetServer, TView>(
	commands: &mut TCommands,
	asset_server: &Res<TAssetServer>,
	visualizer: &Visualizer<TView>,
	key: &'static str,
	model: &Option<ModelPath>,
) -> Result<(), Error>
where
	TCommands: TryComplexInsert<Option<Handle<Scene>>>,
	TAssetServer: Load<ModelPath, Handle<Scene>> + Resource,
{
	let entity = visualizer
		.entities
		.get(&Name::from(key))
		.ok_or(entity_not_found_error(key))?;
	let model = model.as_ref().map(|m| asset_server.load(m));

	commands.try_complex_insert(*entity, model);
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
	use common::{
		simple_init,
		test_tools::utils::new_handle,
		traits::{mock::Mock, nested_mock::NestedMocks, try_complex_insert::TryComplexInsert},
	};
	use macros::NestedMocks;
	use mockall::{automock, mock, predicate::eq};

	enum _Key {
		A,
		B,
	}

	#[derive(Debug, PartialEq)]
	struct _View;

	impl KeyString<_Key> for _View {
		fn key_string(key: &_Key) -> &'static str {
			match key {
				_Key::A => "a",
				_Key::B => "b",
			}
		}
	}

	#[derive(Default)]
	struct _Content {
		uses_view: bool,
		model: Option<ModelPath>,
	}

	impl UsesView<_View> for _Content {
		fn uses_view(&self) -> bool {
			self.uses_view
		}
	}

	impl Getter<Option<ModelPath>> for _Content {
		fn get(&self) -> Option<ModelPath> {
			self.model
		}
	}

	#[test]
	fn add_item() {
		let item = Item {
			content: _Content {
				uses_view: true,
				model: Some(ModelPath("my model")),
			},
			..default()
		};
		let visualize = VisualizeCommands::<_View>::default().with_item(&_Key::A, Some(&item));

		assert_eq!(
			VisualizeCommands::<_View> {
				phantom_data: PhantomData,
				commands: HashMap::from([("a", Some(ModelPath("my model")))]),
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
				commands: HashMap::from([("a", None)]),
			},
			visualize,
		)
	}

	#[test]
	fn add_multiple_items() {
		let item_a = Item {
			content: _Content {
				uses_view: true,
				model: Some(ModelPath("my model a")),
			},
			..default()
		};
		let item_b = Item {
			content: _Content {
				uses_view: true,
				model: Some(ModelPath("my model b")),
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
					("a", Some(ModelPath("my model a"))),
					("b", Some(ModelPath("my model b")))
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
				model: Some(ModelPath("my model")),
			},
			..default()
		};
		let visualize = VisualizeCommands::<_View>::default().with_item(&_Key::A, Some(&item));

		assert_eq!(
			VisualizeCommands::<_View> {
				phantom_data: PhantomData,
				commands: HashMap::from([("a", None as Option<ModelPath>)]),
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

	#[derive(Resource, Default, NestedMocks)]
	struct _AssetServer {
		mock: Mock_AssetServer,
	}

	#[automock]
	impl Load<ModelPath, Handle<Scene>> for _AssetServer {
		fn load(&self, key: &ModelPath) -> Handle<Scene> {
			self.mock.load(key)
		}
	}

	fn setup(asset_server: _AssetServer) -> App {
		let mut app = App::new();
		app.insert_resource(asset_server);
		app
	}

	#[test]
	fn visualize_scene() {
		let handle = new_handle();
		let mut app = setup(_AssetServer::new().with_mock(|mock| {
			mock.expect_load()
				.times(1)
				.with(eq(ModelPath("my model")))
				.return_const(handle.clone());
		}));
		app.world_mut().spawn((
			Visualizer::<_View>::new([(Name::from("a"), Entity::from_raw(42))]),
			VisualizeCommands::<_View> {
				commands: HashMap::from([("a", Some(ModelPath("my model")))]),
				..default()
			},
		));

		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_complex_insert()
				.times(1)
				.with(eq(Entity::from_raw(42)), eq(Some(handle.clone())))
				.return_const(());
			mock.expect_try_remove_from::<VisualizeCommands<_View>>()
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			commands,
			visualize_system::<In<Mock_Commands>, _AssetServer, _View>,
		);
	}

	#[test]
	fn visualize_none() {
		let mut app = setup(_AssetServer::new().with_mock(|mock| {
			mock.expect_load().never().return_const(new_handle());
		}));
		app.world_mut().spawn((
			Visualizer::<_View>::new([(Name::from("a"), Entity::from_raw(42))]),
			VisualizeCommands::<_View> {
				commands: HashMap::from([("a", None)]),
				..default()
			},
		));

		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_complex_insert()
				.times(1)
				.with(eq(Entity::from_raw(42)), eq(None))
				.return_const(());
			mock.expect_try_remove_from::<VisualizeCommands<_View>>()
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			commands,
			visualize_system::<In<Mock_Commands>, _AssetServer, _View>,
		);
	}

	#[test]
	fn remove_visualize_component() {
		let mut app = setup(_AssetServer::new().with_mock(|mock| {
			mock.expect_load().return_const(new_handle());
		}));
		let entity = app
			.world_mut()
			.spawn((
				Visualizer::<_View>::new([(Name::from("a"), Entity::from_raw(42))]),
				VisualizeCommands::<_View> {
					commands: HashMap::from([("a", Some(ModelPath("my model")))]),
					..default()
				},
			))
			.id();

		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_complex_insert().return_const(());
			mock.expect_try_remove_from::<VisualizeCommands<_View>>()
				.times(1)
				.with(eq(entity))
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			commands,
			visualize_system::<In<Mock_Commands>, _AssetServer, _View>,
		);
	}

	#[test]
	fn remove_visualize_component_even_when_commands_empty() {
		let mut app = setup(_AssetServer::new().with_mock(|mock| {
			mock.expect_load().return_const(new_handle());
		}));
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
			mock.expect_try_complex_insert().return_const(());
			mock.expect_try_remove_from::<VisualizeCommands<_View>>()
				.times(1)
				.with(eq(entity))
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			commands,
			visualize_system::<In<Mock_Commands>, _AssetServer, _View>,
		);
	}

	#[test]
	fn return_error_when_key_entity_not_found() {
		let mut app = setup(_AssetServer::new().with_mock(|mock| {
			mock.expect_load().never().return_const(new_handle());
		}));
		app.world_mut().spawn((
			Visualizer::<_View>::new([(Name::from("a"), Entity::from_raw(42))]),
			VisualizeCommands::<_View> {
				commands: HashMap::from([("other key", Some(ModelPath("my model")))]),
				..default()
			},
		));
		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_complex_insert().never().return_const(());
			mock.expect_try_remove_from::<VisualizeCommands<_View>>()
				.return_const(());
		});

		let results = app.world_mut().run_system_once_with(
			commands,
			visualize_system::<In<Mock_Commands>, _AssetServer, _View>,
		);

		assert_eq!(vec![Err(entity_not_found_error("other key"))], results);
	}
}
