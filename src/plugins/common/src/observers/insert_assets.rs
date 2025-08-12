use crate::{
	components::insert_asset::InsertAsset,
	traits::{accessors::get::TryApplyOn, asset_marker::AssetMarker},
	zyheeda_commands::ZyheedaCommands,
};
use bevy::prelude::*;
use std::{any::TypeId, collections::HashMap};

impl<TAsset> InsertAsset<TAsset>
where
	TAsset: AssetMarker,
{
	pub(crate) fn apply(
		trigger: Trigger<OnAdd, Self>,
		mut commands: ZyheedaCommands,
		mut caches: Local<HashMap<TypeId, Handle<TAsset>>>,
		mut assets: ResMut<Assets<TAsset>>,
		components: Query<&Self>,
	) {
		let entity = trigger.target();
		let Ok(component) = components.get(entity) else {
			return;
		};
		let handle = component.get_handle(&mut caches, &mut assets);

		commands.try_apply_on(&entity, |mut e| {
			e.try_remove::<Self>().try_insert(TAsset::component(handle));
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::asset_marker::internal;
	use std::sync::Arc;
	use testing::SingleThreadedApp;
	#[derive(Asset, TypePath, Debug, PartialEq, Default)]
	struct _Asset;

	impl internal::AssetMarker for _Asset {
		type TWrapper = _Wrapper;

		fn wrap(handle: Handle<Self>) -> Self::TWrapper {
			_Wrapper(handle)
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Wrapper(Handle<_Asset>);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<Assets<_Asset>>();
		app.add_observer(InsertAsset::<_Asset>::apply);

		app
	}

	#[test]
	fn add_asset_to_assets() {
		let mut app = setup();

		app.world_mut().spawn(InsertAsset::unique(_Asset::default));

		assert_eq!(1, app.world().resource::<Assets<_Asset>>().iter().count());
	}

	#[test]
	fn insert_wrapped_handle() {
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn(InsertAsset::unique(_Asset::default))
			.id();

		let mut assets = app.world_mut().resource_mut::<Assets<_Asset>>();
		let id = assets.ids().next().unwrap();
		let handle = assets.get_strong_handle(id).unwrap();
		assert_eq!(
			Some(&_Wrapper(handle)),
			app.world().entity(entity).get::<_Wrapper>()
		);
	}

	#[test]
	fn insert_same_if_shared() {
		struct _Marker;

		let mut app = setup();

		let entity_a = app
			.world_mut()
			.spawn(InsertAsset::shared::<_Marker>(_Asset::default))
			.id();
		let entity_b = app
			.world_mut()
			.spawn(InsertAsset::shared::<_Marker>(_Asset::default))
			.id();

		let wrapper_a = app.world().entity(entity_b).get::<_Wrapper>();
		let wrapper_b = app.world().entity(entity_a).get::<_Wrapper>();
		assert!(wrapper_a == wrapper_b);
	}

	#[test]
	fn insert_shared_with_proper_ref_count() {
		struct _Marker;

		let mut app = setup();

		let a = app
			.world_mut()
			.spawn(InsertAsset::shared::<_Marker>(_Asset::default))
			.id();
		let b = app
			.world_mut()
			.spawn(InsertAsset::shared::<_Marker>(_Asset::default))
			.id();

		let Handle::Strong(a) = &app.world().entity(a).get::<_Wrapper>().unwrap().0 else {
			panic!("expected a strong handle");
		};
		let a = Arc::strong_count(a);
		let Handle::Strong(b) = &app.world().entity(b).get::<_Wrapper>().unwrap().0 else {
			panic!("expected a strong handle");
		};
		let b = Arc::strong_count(b);
		assert!(
			a >= 2 && b >= 2,
			"Counts a: {a} vs expected >= 2, Counts b: {b} vs expected >= 2",
		);
	}

	#[test]
	fn insert_unique_if_unique() {
		let mut app = setup();

		let entity_a = app
			.world_mut()
			.spawn(InsertAsset::unique(_Asset::default))
			.id();
		let entity_b = app
			.world_mut()
			.spawn(InsertAsset::unique(_Asset::default))
			.id();

		let wrapper_a = app.world().entity(entity_b).get::<_Wrapper>();
		let wrapper_b = app.world().entity(entity_a).get::<_Wrapper>();
		assert!(wrapper_a != wrapper_b);
	}

	#[test]
	fn insert_correct_shared_when_unique_intermixed() {
		struct _Marker;

		let mut app = setup();

		app.world_mut().spawn(InsertAsset::unique(_Asset::default));
		let entity_a = app
			.world_mut()
			.spawn(InsertAsset::shared::<_Marker>(_Asset::default))
			.id();
		let entity_b = app
			.world_mut()
			.spawn(InsertAsset::shared::<_Marker>(_Asset::default))
			.id();

		let wrapper_a = app.world().entity(entity_b).get::<_Wrapper>();
		let wrapper_b = app.world().entity(entity_a).get::<_Wrapper>();
		assert!(wrapper_a == wrapper_b);
	}

	#[test]
	fn remove_asset_component() {
		let mut app = setup();

		let entity = app
			.world_mut()
			.spawn(InsertAsset::unique(_Asset::default))
			.id();

		assert!({ !app.world().entity(entity).contains::<InsertAsset<_Asset>>() });
	}
}
