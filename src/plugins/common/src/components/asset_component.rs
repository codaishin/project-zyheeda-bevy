use crate::traits::{
	asset_marker::AssetMarker,
	try_insert_on::TryInsertOn,
	try_remove_from::TryRemoveFrom,
};
use bevy::prelude::*;
use std::{any::TypeId, collections::HashMap};

#[derive(Component, Debug)]
/// Defines an asset that should be added to an [`Entity`]
///
/// This is a command like component and will be removed from
/// the [`Entity`] after the corresponding asset has been added.
pub struct AssetComponent<TAsset>
where
	TAsset: AssetMarker,
{
	new_asset: fn() -> TAsset,
	shared: Option<TypeId>,
}

impl<TAsset> PartialEq for AssetComponent<TAsset>
where
	TAsset: AssetMarker,
{
	fn eq(&self, other: &Self) -> bool {
		self.new_asset == other.new_asset && self.shared == other.shared
	}
}

impl<TAsset> AssetComponent<TAsset>
where
	TAsset: AssetMarker,
{
	/// Define an asset, that will be instantiated at runtime.
	///
	/// Uses `new_asset` to create an asset and adds the asset handle via [`AssetMarker::component`]
	/// to the [`Entity`].
	pub fn unique(new_asset: fn() -> TAsset) -> Self {
		Self {
			new_asset,
			shared: None,
		}
	}

	/// Define an asset, that will be instantiated at runtime.
	///
	/// Uses `new_asset` to create an asset and adds the asset handle via [`AssetMarker::component`]
	/// to the [`Entity`].
	///
	/// If a shared asset for `TDriver` already exists, that asset's handle will be used
	/// instead as input for [`AssetMarker::component`] and no new asset will be created.
	///
	/// `TDriver` can be any type, but is usually the component, which "drives" the insertion
	/// of this component, for instance via a `require` attribute.
	pub fn shared<TDriver>(new_asset: fn() -> TAsset) -> Self
	where
		TDriver: 'static,
	{
		Self {
			new_asset,
			shared: Some(TypeId::of::<TDriver>()),
		}
	}

	/// Define an asset, that will be instantiated at runtime.
	///
	/// Uses `new_asset` to create an asset and adds the asset handle via [`AssetMarker::component`]
	/// to the [`Entity`].
	///
	/// If a shared asset for `type_id` already exists, that asset's handle will be used
	/// instead as input for [`AssetMarker::component`] and no new asset will be created.
	///
	/// `TypeId` can be any type, but is usually the id of a  component, which "drives" the insertion
	/// of this component, for instance via a `require` attribute.
	pub fn shared_id(new_asset: fn() -> TAsset, type_id: TypeId) -> Self {
		Self {
			new_asset,
			shared: Some(type_id),
		}
	}

	pub(crate) fn add_asset(
		mut commands: Commands,
		mut caches: Local<HashMap<TypeId, Handle<TAsset>>>,
		mut assets: ResMut<Assets<TAsset>>,
		components: Query<(Entity, &Self)>,
	) {
		for (entity, component) in &components {
			let handle = component.get_handle(&mut caches, &mut assets);

			commands.try_insert_on(entity, TAsset::component(handle));
			commands.try_remove_from::<Self>(entity);
		}
	}

	fn get_handle(
		&self,
		caches: &mut HashMap<TypeId, Handle<TAsset>>,
		assets: &mut Assets<TAsset>,
	) -> Handle<TAsset>
	where
		TAsset: AssetMarker,
	{
		let Some(shared_id) = self.shared else {
			return assets.add(self.create_asset());
		};

		let Some(handle) = caches.get(&shared_id) else {
			let handle = assets.add(self.create_asset());
			caches.insert(shared_id, handle.clone());
			return handle;
		};

		handle.clone()
	}

	fn create_asset(&self) -> TAsset {
		(self.new_asset)()
	}
}

#[derive(Component, Debug, PartialEq)]
/// Defines an asset that should be added to an [`Entity`]
///
/// This is a command like component and will be removed from
/// the [`Entity`] after the corresponding asset has been added.
pub struct AssetComponentFromSource<TAsset, TSource>
where
	TAsset: AssetMarker,
	TSource: Component,
{
	new_asset: fn(&TSource) -> TAsset,
	shared: Option<TypeId>,
}

impl<TAsset, TSource> AssetComponentFromSource<TAsset, TSource>
where
	TAsset: AssetMarker,
	TSource: Component + 'static,
{
	/// Define an asset, that will be instantiated at runtime.
	///
	/// Uses `new_asset` to create an asset from a `TSource` component on the same [`Entity`]
	/// and adds the asset handle via [`AssetMarker::component`] to the [`Entity`].
	///
	/// <div class="warning">
	///   Only works, if [`Self::add_asset`] system has been registered
	/// </div>
	pub fn unique(new_asset: fn(&TSource) -> TAsset) -> Self {
		Self {
			new_asset,
			shared: None,
		}
	}

	/// Define an asset, that will be instantiated at runtime.
	///
	/// Uses `new_asset` to create an asset from a `TSource` component on the same [`Entity`]
	/// and adds the asset handle via [`AssetMarker::component`] to the [`Entity`].
	///
	/// If a shared asset for `TSource` already exists, that asset's handle will be used
	/// instead and no new asset will be created.
	///
	/// <div class="warning">
	///   Only works, if [`Self::add_asset`] system has been registered
	/// </div>
	pub fn shared(new_asset: fn(&TSource) -> TAsset) -> Self {
		Self {
			new_asset,
			shared: Some(TypeId::of::<TSource>()),
		}
	}

	pub fn add_asset(
		mut commands: Commands,
		mut caches: Local<HashMap<TypeId, Handle<TAsset>>>,
		mut assets: ResMut<Assets<TAsset>>,
		components: Query<(Entity, &Self, &TSource)>,
	) {
		for (entity, component, source) in &components {
			let handle = component.get_handle_from_source(&mut caches, &mut assets, source);

			commands.try_insert_on(entity, TAsset::component(handle));
			commands.try_remove_from::<Self>(entity);
		}
	}

	fn get_handle_from_source(
		&self,
		caches: &mut HashMap<TypeId, Handle<TAsset>>,
		assets: &mut Assets<TAsset>,
		source: &TSource,
	) -> Handle<TAsset>
	where
		TAsset: AssetMarker,
	{
		let Some(shared_id) = self.shared else {
			return assets.add(self.create_asset_from(source));
		};

		let Some(handle) = caches.get(&shared_id) else {
			let handle = assets.add(self.create_asset_from(source));
			caches.insert(shared_id, handle.clone());
			return handle;
		};

		handle.clone()
	}

	fn create_asset_from(&self, source: &TSource) -> TAsset {
		(self.new_asset)(source)
	}
}

#[cfg(test)]
mod test_add_asset {
	use std::sync::Arc;

	use super::*;
	use crate::{test_tools::utils::SingleThreadedApp, traits::asset_marker::internal};

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
			.spawn(AssetComponent::shared::<_Marker>(_Asset::default))
			.id();
		let entity_b = app
			.world_mut()
			.spawn(AssetComponent::shared::<_Marker>(_Asset::default))
			.id();

		app.update();

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
			.spawn(AssetComponent::shared::<_Marker>(_Asset::default))
			.id();
		let b = app
			.world_mut()
			.spawn(AssetComponent::shared::<_Marker>(_Asset::default))
			.id();

		app.update();

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
			.spawn(AssetComponent::unique(_Asset::default))
			.id();
		let entity_b = app
			.world_mut()
			.spawn(AssetComponent::unique(_Asset::default))
			.id();

		app.update();

		let wrapper_a = app.world().entity(entity_b).get::<_Wrapper>();
		let wrapper_b = app.world().entity(entity_a).get::<_Wrapper>();
		assert!(wrapper_a != wrapper_b);
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

		let wrapper_a = app.world().entity(entity_b).get::<_Wrapper>();
		let wrapper_b = app.world().entity(entity_a).get::<_Wrapper>();
		assert!(wrapper_a == wrapper_b);
	}

	#[test]
	fn remove_asset_component() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(AssetComponent::unique(_Asset::default))
			.id();

		app.update();

		assert!({
			!app.world()
				.entity(entity)
				.contains::<AssetComponent<_Asset>>()
		});
	}
}

#[cfg(test)]
mod test_add_asset_from_source {
	use std::sync::Arc;

	use super::*;
	use crate::{test_tools::utils::SingleThreadedApp, traits::asset_marker::internal};

	#[derive(Component, TypePath, Debug, PartialEq, Clone)]
	struct _Source;

	#[derive(Asset, TypePath, Debug, PartialEq)]
	struct _Asset(_Source);

	impl _Asset {
		fn from_source(source: &_Source) -> _Asset {
			_Asset(source.clone())
		}
	}

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
		app.add_systems(
			Update,
			AssetComponentFromSource::<_Asset, _Source>::add_asset,
		);

		app
	}

	#[test]
	fn add_asset_to_assets() {
		let mut app = setup();
		app.world_mut().spawn((
			AssetComponentFromSource::unique(_Asset::from_source),
			_Source,
		));

		app.update();

		assert_eq!(1, app.world().resource::<Assets<_Asset>>().iter().count());
	}

	#[test]
	fn insert_wrapped_handle() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				AssetComponentFromSource::unique(_Asset::from_source),
				_Source,
			))
			.id();

		app.update();

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
		let mut app = setup();
		let entity_a = app
			.world_mut()
			.spawn((
				AssetComponentFromSource::shared(_Asset::from_source),
				_Source,
			))
			.id();
		let entity_b = app
			.world_mut()
			.spawn((
				AssetComponentFromSource::shared(_Asset::from_source),
				_Source,
			))
			.id();

		app.update();

		let handle_a = app.world().entity(entity_b).get::<_Wrapper>();
		let handle_b = app.world().entity(entity_a).get::<_Wrapper>();
		assert!(handle_a == handle_b);
	}

	#[test]
	fn insert_shared_with_proper_ref_count() {
		let mut app = setup();
		let a = app
			.world_mut()
			.spawn((
				AssetComponentFromSource::shared(_Asset::from_source),
				_Source,
			))
			.id();
		let b = app
			.world_mut()
			.spawn((
				AssetComponentFromSource::shared(_Asset::from_source),
				_Source,
			))
			.id();

		app.update();

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
			.spawn((
				AssetComponentFromSource::unique(_Asset::from_source),
				_Source,
			))
			.id();
		let entity_b = app
			.world_mut()
			.spawn((
				AssetComponentFromSource::unique(_Asset::from_source),
				_Source,
			))
			.id();

		app.update();

		let handle_a = app.world().entity(entity_b).get::<_Wrapper>();
		let handle_b = app.world().entity(entity_a).get::<_Wrapper>();
		assert!(handle_a != handle_b);
	}

	#[test]
	fn insert_correct_shared_when_unique_intermixed() {
		let mut app = setup();
		app.world_mut().spawn((
			AssetComponentFromSource::unique(_Asset::from_source),
			_Source,
		));
		let entity_a = app
			.world_mut()
			.spawn((
				AssetComponentFromSource::shared(_Asset::from_source),
				_Source,
			))
			.id();
		let entity_b = app
			.world_mut()
			.spawn((
				AssetComponentFromSource::shared(_Asset::from_source),
				_Source,
			))
			.id();

		app.update();

		let handle_a = app.world().entity(entity_b).get::<_Wrapper>();
		let handle_b = app.world().entity(entity_a).get::<_Wrapper>();
		assert!(handle_a == handle_b);
	}

	#[test]
	fn remove_asset_component() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				AssetComponentFromSource::unique(_Asset::from_source),
				_Source,
			))
			.id();

		app.update();

		assert!({
			!app.world()
				.entity(entity)
				.contains::<AssetComponent<_Asset>>()
		});
	}
}
