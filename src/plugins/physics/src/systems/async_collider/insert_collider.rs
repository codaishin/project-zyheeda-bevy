use crate::components::async_collider::{AsyncCollider, ColliderType, Source};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	errors::{ErrorData, Level},
	traits::{
		accessors::get::{GetMut, TryApplyOn},
		load_asset::LoadAsset,
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::fmt::Display;

impl AsyncCollider {
	pub(crate) fn insert_collider(
		colliders: Query<(Entity, &mut Self, Option<&Mesh3d>)>,
		commands: ZyheedaCommands,
		server: ResMut<AssetServer>,
		meshes: Res<Assets<Mesh>>,
	) -> Result<(), Vec<InsertColliderError>> {
		Self::insert_collider_via::<AssetServer, Collider>(colliders, commands, server, meshes)
	}

	fn insert_collider_via<TAssetServer, TCollider>(
		mut colliders: Query<(Entity, &mut Self, Option<&Mesh3d>)>,
		mut commands: ZyheedaCommands,
		mut server: ResMut<TAssetServer>,
		meshes: Res<Assets<Mesh>>,
	) -> Result<(), Vec<InsertColliderError>>
	where
		TAssetServer: Resource + LoadAsset,
		TCollider: Component + ConvexMeshCollider + ConcaveMeshCollider,
	{
		let mut errors = vec![];

		for (entity, mut async_collider, mesh) in &mut colliders {
			match (&async_collider.source, mesh) {
				(Source::Path(path), ..) => {
					async_collider.source = Source::Handle(server.load_asset(*path));
				}
				(Source::MeshOfEntity, Some(Mesh3d(handle))) => {
					async_collider.source = Source::Handle(handle.clone());
				}
				(Source::MeshOfEntity, None) => {
					errors.push(InsertColliderError::MeshMissing(entity));
					commands.try_apply_on(&entity, |mut e| {
						e.try_remove::<Self>();
					});
				}
				(Source::Handle(handle), ..) => {
					let Some(mesh) = meshes.get(handle) else {
						continue;
					};

					let Some(mut entity) = commands.get_mut(&entity) else {
						continue;
					};

					entity.try_remove::<Self>();

					let Some(collider) = async_collider.get_mesh_collider::<TCollider>(mesh) else {
						errors.push(InsertColliderError::CannotCompute(handle.clone()));
						continue;
					};

					entity.try_insert(collider);

					let Some(scale) = async_collider.scale else {
						continue;
					};

					entity.try_insert(scale);
				}
			}
		}

		if !errors.is_empty() {
			return Err(errors);
		}

		Ok(())
	}

	fn get_mesh_collider<TCollider>(&self, mesh: &Mesh) -> Option<TCollider>
	where
		TCollider: ConvexMeshCollider + ConcaveMeshCollider,
	{
		match self.collider_type {
			ColliderType::Convex => TCollider::convex_mesh_collider(mesh),
			ColliderType::Concave => TCollider::concave_mesh_collider(mesh),
		}
	}
}

pub(crate) trait ConvexMeshCollider: Sized {
	fn convex_mesh_collider(mesh: &Mesh) -> Option<Self>;
}

impl ConvexMeshCollider for Collider {
	fn convex_mesh_collider(mesh: &Mesh) -> Option<Self> {
		Collider::from_bevy_mesh(mesh, &ComputedColliderShape::ConvexHull)
	}
}

pub(crate) trait ConcaveMeshCollider: Sized {
	fn concave_mesh_collider(mesh: &Mesh) -> Option<Self>;
}

impl ConcaveMeshCollider for Collider {
	fn concave_mesh_collider(mesh: &Mesh) -> Option<Self> {
		Collider::from_bevy_mesh(
			mesh,
			&ComputedColliderShape::TriMesh(
				TriMeshFlags::FIX_INTERNAL_EDGES
					| TriMeshFlags::MERGE_DUPLICATE_VERTICES
					| TriMeshFlags::DELETE_DEGENERATE_TRIANGLES,
			),
		)
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum InsertColliderError {
	CannotCompute(Handle<Mesh>),
	MeshMissing(Entity),
}

impl Display for InsertColliderError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			InsertColliderError::CannotCompute(mesh) => {
				write!(f, "{mesh:?}: cannot insert collider")
			}
			InsertColliderError::MeshMissing(entity) => {
				write!(f, "{entity:?}: has no mesh")
			}
		}
	}
}

impl ErrorData for InsertColliderError {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl std::fmt::Display {
		"Insert Collider Error"
	}

	fn into_details(self) -> impl std::fmt::Display {
		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::asset::uuid::uuid;
	use common::traits::load_asset::mock::MockAssetServer;
	use std::marker::PhantomData;
	use test_case::test_case;
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Component, Debug, PartialEq)]
	enum _Collider {
		Convex(Mesh),
		Concave(Mesh),
	}

	impl ConvexMeshCollider for _Collider {
		fn convex_mesh_collider(mesh: &Mesh) -> Option<Self> {
			Some(_Collider::Convex(mesh.clone()))
		}
	}

	impl ConcaveMeshCollider for _Collider {
		fn concave_mesh_collider(mesh: &Mesh) -> Option<Self> {
			Some(_Collider::Concave(mesh.clone()))
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(Result<(), Vec<InsertColliderError>>);

	fn setup<TCollider>(assets: &[(Handle<Mesh>, Mesh)], server: MockAssetServer) -> App
	where
		TCollider: Component + ConvexMeshCollider + ConcaveMeshCollider,
	{
		let mut app = App::new().single_threaded(Update);
		let mut asset_resource = Assets::default();

		for (id, asset) in assets {
			_ = asset_resource.insert(id, asset.clone());
		}

		app.insert_resource(asset_resource);
		app.insert_resource(server);
		app.add_systems(
			Update,
			AsyncCollider::insert_collider_via::<MockAssetServer, TCollider>.pipe(
				|In(result), mut c: Commands| {
					c.insert_resource(_Result(result));
				},
			),
		);

		app
	}

	#[test]
	fn set_handle_from_path() {
		let handle = new_handle::<Mesh>();
		let mut app = setup::<_Collider>(
			&[],
			MockAssetServer::default()
				.path("my/path")
				.returns(handle.clone()),
		);
		let entity = app
			.world_mut()
			.spawn(AsyncCollider::concave("my/path"))
			.id();

		app.update();

		assert_eq!(
			Some(&AsyncCollider::concave(handle)),
			app.world().entity(entity).get::<AsyncCollider>(),
		);
	}

	#[test]
	fn set_handle_from_entity() {
		let handle = new_handle::<Mesh>();
		let mut app = setup::<_Collider>(&[], MockAssetServer::default());
		let entity = app
			.world_mut()
			.spawn((
				AsyncCollider::concave(Source::MeshOfEntity),
				Mesh3d(handle.clone()),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&AsyncCollider::concave(handle)),
			app.world().entity(entity).get::<AsyncCollider>(),
		);
	}

	#[test]
	fn insert_convex_collider() {
		let handle = new_handle::<Mesh>();
		let mesh = Mesh::from(Cuboid::new(1., 2., 3.));
		let mut app = setup::<_Collider>(&[(handle.clone(), mesh)], MockAssetServer::default());
		let entity = app.world_mut().spawn(AsyncCollider::convex(handle)).id();

		app.update();

		assert_eq!(
			Some(&_Collider::Convex(Mesh::from(Cuboid::new(1., 2., 3.)))),
			app.world().entity(entity).get::<_Collider>(),
		);
	}

	#[test]
	fn insert_concave_collider() {
		let handle = new_handle::<Mesh>();
		let mesh = Mesh::from(Cuboid::new(1., 2., 3.));
		let mut app = setup::<_Collider>(&[(handle.clone(), mesh)], MockAssetServer::default());
		let entity = app.world_mut().spawn(AsyncCollider::concave(handle)).id();

		app.update();

		assert_eq!(
			Some(&_Collider::Concave(Mesh::from(Cuboid::new(1., 2., 3.)))),
			app.world().entity(entity).get::<_Collider>(),
		);
	}

	#[test]
	fn insert_collider_scale() {
		let handle = new_handle::<Mesh>();
		let mesh = Mesh::from(Cuboid::new(1., 2., 3.));
		let mut app = setup::<_Collider>(&[(handle.clone(), mesh)], MockAssetServer::default());
		let entity = app
			.world_mut()
			.spawn(
				AsyncCollider::concave(handle)
					.with_scale(ColliderScale::Absolute(Vec3::new(1., 2., 3.))),
			)
			.id();

		app.update();

		assert_eq!(
			Some(&ColliderScale::Absolute(Vec3::new(1., 2., 3.))),
			app.world().entity(entity).get::<ColliderScale>(),
		);
	}

	const MESH_EXISTS: Handle<Mesh> =
		Handle::Uuid(uuid!("b178e816-93c6-430b-98b8-fca3116a7b58"), PhantomData);
	const MESH_DOES_NOT_EXIST: Handle<Mesh> =
		Handle::Uuid(uuid!("7342b7fa-2742-4688-94c8-82436c0d4d8a"), PhantomData);

	#[test_case(AsyncCollider::concave("my/path"), true; "not when only path")]
	#[test_case(AsyncCollider::concave(Source::MeshOfEntity), true; "not when set to read entity mesh")]
	#[test_case(AsyncCollider::concave(MESH_DOES_NOT_EXIST), true; "not when mesh missing")]
	#[test_case(AsyncCollider::concave(MESH_EXISTS), false; "when mesh exists")]
	fn remove_async_collider(collider: AsyncCollider, is_present: bool) {
		let mut app = setup::<_Collider>(
			&[(MESH_EXISTS, Mesh::from(Sphere::new(1.)))],
			MockAssetServer::default(),
		);
		let entity = app.world_mut().spawn((collider, Mesh3d::default())).id();

		app.update();

		assert_eq!(
			is_present,
			app.world().entity(entity).contains::<AsyncCollider>(),
		);
	}

	#[test_case(ColliderType::Convex; "convex")]
	#[test_case(ColliderType::Concave; "concave")]
	fn return_missing_mesh_error(collider_type: ColliderType) {
		let mut app = setup::<_Collider>(&[], MockAssetServer::default());
		let entity = app
			.world_mut()
			.spawn(AsyncCollider {
				source: Source::MeshOfEntity,
				collider_type,
				scale: None,
			})
			.id();

		app.update();

		assert_eq!(
			&_Result(Err(vec![InsertColliderError::MeshMissing(entity)])),
			app.world().resource::<_Result>(),
		);
	}

	#[derive(Component)]
	struct _FaultyCollider;

	impl ConvexMeshCollider for _FaultyCollider {
		fn convex_mesh_collider(_: &Mesh) -> Option<Self> {
			None
		}
	}

	impl ConcaveMeshCollider for _FaultyCollider {
		fn concave_mesh_collider(_: &Mesh) -> Option<Self> {
			None
		}
	}

	#[test_case(ColliderType::Convex; "convex")]
	#[test_case(ColliderType::Concave; "concave")]
	fn return_compute_error(collider_type: ColliderType) {
		let handle = new_handle();
		let mut app = setup::<_FaultyCollider>(
			&[(handle.clone(), Mesh::from(Sphere::new(1.)))],
			MockAssetServer::default(),
		);
		app.world_mut().spawn(AsyncCollider {
			source: Source::Handle(handle.clone()),
			collider_type,
			scale: None,
		});

		app.update();

		assert_eq!(
			&_Result(Err(vec![InsertColliderError::CannotCompute(handle)])),
			app.world().resource::<_Result>(),
		);
	}

	#[test]
	fn remove_collider_on_mesh_missing_error() {
		let mut app = setup::<_FaultyCollider>(&[], MockAssetServer::default());
		let entity = app
			.world_mut()
			.spawn(AsyncCollider::concave(Source::MeshOfEntity))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<AsyncCollider>());
	}

	#[test]
	fn remove_collider_on_compute_failure() {
		let handle = new_handle();
		let mut app = setup::<_FaultyCollider>(
			&[(handle.clone(), Mesh::from(Sphere::new(1.)))],
			MockAssetServer::default(),
		);
		let entity = app.world_mut().spawn(AsyncCollider::concave(handle)).id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<AsyncCollider>());
	}

	#[test]
	fn return_ok() {
		let handle = new_handle();
		let mut app = setup::<_Collider>(
			&[(handle.clone(), Mesh::from(Sphere::new(1.)))],
			MockAssetServer::default(),
		);
		app.world_mut().spawn(AsyncCollider::concave(handle));

		app.update();

		assert_eq!(&_Result(Ok(())), app.world().resource::<_Result>());
	}
}
