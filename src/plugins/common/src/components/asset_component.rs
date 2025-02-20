use crate::traits::{
	asset_marker::AssetMarker,
	try_insert_on::TryInsertOn,
	try_remove_from::TryRemoveFrom,
};
use bevy::prelude::*;
use std::{any::TypeId, collections::HashMap};

#[derive(Component, Debug, PartialEq)]
pub struct AssetComponent<TAsset>
where
	TAsset: AssetMarker,
{
	new_asset: fn() -> TAsset,
	shared: Option<TypeId>,
}

impl<TAsset> AssetComponent<TAsset>
where
	TAsset: AssetMarker,
{
	pub fn unique(new_asset: fn() -> TAsset) -> Self {
		Self {
			new_asset,
			shared: None,
		}
	}

	pub fn shared<TSharedMarker>(new_asset: fn() -> TAsset) -> Self
	where
		TSharedMarker: 'static,
	{
		Self {
			new_asset,
			shared: Some(TypeId::of::<TSharedMarker>()),
		}
	}

	pub(crate) fn add_asset(
		mut commands: Commands,
		mut shares: Local<HashMap<TypeId, AssetId<TAsset>>>,
		mut assets: ResMut<Assets<TAsset>>,
		components: Query<(Entity, &Self)>,
	) {
		for (entity, component) in &components {
			let handle = component.get_handle(&mut shares, &mut assets);

			commands.try_insert_on(entity, TAsset::wrap(handle));
			commands.try_remove_from::<Self>(entity);
		}
	}

	fn create_asset(&self) -> TAsset {
		(self.new_asset)()
	}

	fn get_handle(
		&self,
		shares: &mut HashMap<TypeId, AssetId<TAsset>>,
		assets: &mut Assets<TAsset>,
	) -> Handle<TAsset>
	where
		TAsset: AssetMarker,
	{
		let Some(shared_id) = self.shared else {
			return assets.add(self.create_asset());
		};

		let Some(id) = shares.get(&shared_id) else {
			let handle = assets.add(self.create_asset());
			shares.insert(shared_id, handle.id());
			return handle;
		};

		let Some(handle) = assets.get_strong_handle(*id) else {
			return assets.add(self.create_asset());
		};

		handle
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{test_tools::utils::SingleThreadedApp, traits::asset_marker::internal};

	#[derive(Asset, TypePath, Debug, PartialEq, Default, Clone)]
	struct _Asset;

	impl internal::AssetMarker for _Asset {
		type TComponent = _Wrapper;

		fn wrap(handle: Handle<Self>) -> Self::TComponent {
			_Wrapper(handle)
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Wrapper(Handle<_Asset>);

	impl From<&_Wrapper> for AssetId<_Asset> {
		fn from(component: &_Wrapper) -> Self {
			component.0.id()
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.init_resource::<Assets<_Asset>>();
		app.add_systems(Update, AssetComponent::<_Asset>::add_asset);

		app
	}

	#[test]
	fn add_asset_to_assets() {
		let mut app = setup();
		app.world_mut()
			.spawn(AssetComponent::unique(_Asset::default));

		app.update();

		assert_eq!(1, app.world().resource::<Assets<_Asset>>().iter().count());
	}

	#[test]
	fn insert_wrapped_handle() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(AssetComponent::unique(_Asset::default))
			.id();

		app.update();

		let assets = app.world().resource::<Assets<_Asset>>();
		let id = assets.iter().next().unwrap().0;
		assert_eq!(
			Some(id),
			app.world()
				.entity(entity)
				.get::<_Wrapper>()
				.map(AssetId::<_Asset>::from)
		);
	}

	#[test]
	fn insert_same_if_shared() {
		struct _Marker;

		let mut app = setup();
		let entity_a = app
			.world_mut()
			.spawn(AssetComponent::shared::<_Marker>(_Asset::default))
			.id();
		let entity_b = app
			.world_mut()
			.spawn(AssetComponent::shared::<_Marker>(_Asset::default))
			.id();

		app.update();

		let id_a = app
			.world()
			.entity(entity_b)
			.get::<_Wrapper>()
			.map(AssetId::<_Asset>::from);
		let id_b = app
			.world()
			.entity(entity_a)
			.get::<_Wrapper>()
			.map(AssetId::<_Asset>::from);
		assert!(id_a == id_b);
	}

	#[test]
	fn insert_unique_if_unique() {
		let mut app = setup();
		let entity_a = app
			.world_mut()
			.spawn(AssetComponent::unique(_Asset::default))
			.id();
		let entity_b = app
			.world_mut()
			.spawn(AssetComponent::unique(_Asset::default))
			.id();

		app.update();

		let id_a = app
			.world()
			.entity(entity_b)
			.get::<_Wrapper>()
			.map(AssetId::<_Asset>::from);
		let id_b = app
			.world()
			.entity(entity_a)
			.get::<_Wrapper>()
			.map(AssetId::<_Asset>::from);
		assert!(id_a != id_b);
	}

	#[test]
	fn insert_correct_shared_when_unique_intermixed() {
		struct _Marker;

		let mut app = setup();
		app.world_mut()
			.spawn(AssetComponent::unique(_Asset::default));
		let entity_a = app
			.world_mut()
			.spawn(AssetComponent::shared::<_Marker>(_Asset::default))
			.id();
		let entity_b = app
			.world_mut()
			.spawn(AssetComponent::shared::<_Marker>(_Asset::default))
			.id();

		app.update();

		let id_a = app
			.world()
			.entity(entity_b)
			.get::<_Wrapper>()
			.map(AssetId::<_Asset>::from);
		let id_b = app
			.world()
			.entity(entity_a)
			.get::<_Wrapper>()
			.map(AssetId::<_Asset>::from);
		assert!(id_a == id_b);
	}

	#[test]
	fn insert_new_when_previous_shared_handle_missing() {
		struct _Marker;

		let mut app = setup();
		app.world_mut()
			.spawn(AssetComponent::shared::<_Marker>(_Asset::default));

		app.update();
		let mut assets = app.world_mut().resource_mut::<Assets<_Asset>>();
		let id = assets.ids().next().unwrap();
		assets.remove(id);
		app.update();
		let entity_b = app
			.world_mut()
			.spawn(AssetComponent::shared::<_Marker>(_Asset::default))
			.id();
		app.update();

		assert!(app.world().entity(entity_b).contains::<_Wrapper>());
	}

	#[test]
	fn remove_asset_component() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(AssetComponent::unique(_Asset::default))
			.id();

		app.update();

		assert_eq!(
			None,
			app.world().entity(entity).get::<AssetComponent::<_Asset>>()
		);
	}
}
