use crate::components::visualize::Visualize;
use bevy::{ecs::query::QueryFilter, prelude::*};
use common::{
	errors::{Error, Level},
	traits::{
		accessors::get::GetRef,
		iteration::IterFinite,
		register_visualization::ContainsVisibleItemAssets,
		try_insert_on::TryInsertOn,
	},
};

impl<TComponent> ApplyVisualization for TComponent {}

pub(crate) trait ApplyVisualization {
	fn apply_visualization<TMarker>(
		commands: Commands,
		assets: Res<Assets<Self::TAsset>>,
		visualizers: Query<(&Self, &Visualize<Self, TMarker>), Changed<Self>>,
	) -> Vec<Result<(), Error>>
	where
		Self: ContainsVisibleItemAssets<TMarker> + Component + Sized,
		Self::TKey: IterFinite,
		TMarker: Sync + Send + 'static,
	{
		visualize_system(commands, assets, visualizers)
	}
}

fn visualize_system<TCommands, TAssets, TComponent, TMarker, TFilter>(
	mut commands: TCommands,
	assets: Res<TAssets>,
	components: Query<(&TComponent, &Visualize<TComponent, TMarker>), TFilter>,
) -> Vec<Result<(), Error>>
where
	TCommands: TryInsertOn,
	TAssets: GetRef<Handle<TComponent::TAsset>, TComponent::TAsset> + Resource,
	TComponent: ContainsVisibleItemAssets<TMarker> + Component,
	TComponent::TKey: IterFinite,
	TMarker: Sync + Send + 'static,
	TFilter: QueryFilter,
{
	let mut errors = vec![];

	for (container, visualize) in &components {
		for key in TComponent::TKey::iterator() {
			let asset = container.get_asset(&key, assets.as_ref());
			let bundle = TComponent::visualization_component(asset);
			let result = apply(&mut commands, visualize, key, bundle);
			let Err(error) = result else {
				continue;
			};
			errors.push(Err(error));
		}
	}

	errors
}

fn apply<TCommands, TComponent, TMarker>(
	commands: &mut TCommands,
	visualizer: &Visualize<TComponent, TMarker>,
	key: TComponent::TKey,
	bundle: impl Bundle,
) -> Result<(), Error>
where
	TCommands: TryInsertOn,
	TComponent: ContainsVisibleItemAssets<TMarker>,
{
	let key = TComponent::visualization_entity_name(&key);
	let entity = visualizer
		.entities
		.get(&Name::from(key))
		.ok_or(entity_not_found_error(key))?;

	commands.try_insert_on(*entity, bundle);
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
	use bevy::{ecs::system::RunSystemOnce, utils::HashMap};
	use common::{
		simple_init,
		test_tools::utils::new_handle,
		traits::{
			get_asset::GetAsset,
			iteration::{Iter, IterFinite},
			mock::Mock,
			nested_mock::NestedMocks,
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, mock, predicate::eq};

	#[derive(Debug, PartialEq, Clone, Copy, Eq, Hash)]
	enum _Key {
		A,
		B,
	}

	impl IterFinite for _Key {
		fn iterator() -> Iter<Self> {
			Iter(Some(_Key::A))
		}

		fn next(Iter(current): &Iter<Self>) -> Option<Self> {
			match current.as_ref()? {
				_Key::A => Some(_Key::B),
				_Key::B => None,
			}
		}
	}

	#[derive(Asset, TypePath, Clone)]
	struct _Asset;

	#[derive(Component)]
	struct _Component(HashMap<_Key, Handle<_Asset>>);

	impl _Component {
		fn new<const N: usize>(handles: [(_Key, Handle<_Asset>); N]) -> Self {
			_Component(handles.into())
		}
	}

	impl GetAsset for _Component {
		type TKey = _Key;
		type TAsset = _Asset;

		fn get_asset<'a, TAssets>(
			&'a self,
			key: &Self::TKey,
			assets: &'a TAssets,
		) -> Option<&'a Self::TAsset>
		where
			TAssets: GetRef<Handle<Self::TAsset>, Self::TAsset>,
		{
			let _Component(handles) = self;
			let handle = handles.get(key)?;
			assets.get(handle)
		}
	}

	struct _Marker;

	impl ContainsVisibleItemAssets<_Marker> for _Component {
		type TVisualizationEntityConstraint = ();

		fn visualization_entity_name(key: &Self::TKey) -> &'static str {
			match key {
				_Key::A => "a",
				_Key::B => "b",
			}
		}

		fn visualization_component(_: Option<&Self::TAsset>) -> impl Bundle {
			_Visualize
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Visualize;

	#[derive(Resource, NestedMocks)]
	struct _Assets {
		mock: Mock_Assets,
	}

	#[automock]
	impl GetRef<Handle<_Asset>, _Asset> for _Assets {
		fn get<'a>(&'a self, key: &Handle<_Asset>) -> Option<&'a _Asset> {
			self.mock.get(key)
		}
	}

	mock! {
		_Commands {}
		impl TryInsertOn for _Commands {
			fn try_insert_on<TBundle: Bundle>(&mut self, entity: Entity, bundle: TBundle);
		}
	}

	simple_init!(Mock_Commands);

	fn setup(assets: _Assets) -> App {
		let mut app = App::new();
		app.insert_resource(assets);

		app
	}

	#[test]
	fn insert_visualize_component() {
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_get().return_const(&_Asset);
		}));
		app.world_mut().spawn((
			Visualize::<_Component, _Marker>::new([(Name::from("a"), Entity::from_raw(42))]),
			_Component::new([(_Key::A, new_handle())]),
		));
		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_insert_on()
				.times(1)
				.with(eq(Entity::from_raw(42)), eq(_Visualize))
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			commands,
			visualize_system::<In<Mock_Commands>, _Assets, _Component, _Marker, ()>,
		);
	}

	#[test]
	fn insert_visualize_component_on_different_target() {
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_get().return_const(&_Asset);
		}));
		app.world_mut().spawn((
			Visualize::<_Component, _Marker>::new([(Name::from("b"), Entity::from_raw(42))]),
			_Component::new([(_Key::B, new_handle())]),
		));
		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_insert_on()
				.times(1)
				.with(eq(Entity::from_raw(42)), eq(_Visualize))
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			commands,
			visualize_system::<In<Mock_Commands>, _Assets, _Component, _Marker, ()>,
		);
	}

	#[test]
	fn use_correct_handle() {
		let handle = new_handle();
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_get()
				.with(eq(handle.clone()))
				.times(1)
				.return_const(&_Asset);
		}));
		app.world_mut().spawn((
			Visualize::<_Component, _Marker>::new([(Name::from("a"), Entity::from_raw(42))]),
			_Component::new([(_Key::A, handle)]),
		));
		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_insert_on::<_Visualize>().return_const(());
		});

		app.world_mut().run_system_once_with(
			commands,
			visualize_system::<In<Mock_Commands>, _Assets, _Component, _Marker, ()>,
		);
	}

	#[test]
	fn apply_system_filter() {
		#[derive(Component)]
		struct _Ignore;

		type _Filter = Without<_Ignore>;

		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_get().never().return_const(&_Asset);
		}));
		app.world_mut().spawn((
			Visualize::<_Component, _Marker>::new([(Name::from("a"), Entity::from_raw(42))]),
			_Component::new([(_Key::A, new_handle())]),
			_Ignore,
		));
		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_insert_on::<_Visualize>()
				.never()
				.return_const(());
		});

		app.world_mut().run_system_once_with(
			commands,
			visualize_system::<In<Mock_Commands>, _Assets, _Component, _Marker, _Filter>,
		);
	}

	#[test]
	fn return_error_when_key_entity_not_found() {
		let mut app = setup(_Assets::new().with_mock(|mock| {
			mock.expect_get().return_const(&_Asset);
		}));
		app.world_mut().spawn((
			Visualize::<_Component, _Marker>::new([]),
			_Component::new([]),
		));
		let commands = Mock_Commands::new_mock(|mock| {
			mock.expect_try_insert_on::<_Visualize>().return_const(());
		});

		let results = app.world_mut().run_system_once_with(
			commands,
			visualize_system::<In<Mock_Commands>, _Assets, _Component, _Marker, ()>,
		);

		assert_eq!(
			vec![
				Err(entity_not_found_error("a")),
				Err(entity_not_found_error("b"))
			],
			results
		);
	}
}
