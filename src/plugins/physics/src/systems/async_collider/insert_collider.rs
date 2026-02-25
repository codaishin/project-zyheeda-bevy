use crate::components::async_collider::{AsyncCollider, ColliderType};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	errors::{ErrorData, Level},
	traits::{accessors::get::GetMut, load_asset::LoadAsset},
	zyheeda_commands::ZyheedaCommands,
};
use std::fmt::Display;

impl AsyncCollider {
	pub(crate) fn insert_collider(
		colliders: Query<(Entity, &mut Self)>,
		commands: ZyheedaCommands,
		server: ResMut<AssetServer>,
		meshes: Res<Assets<Mesh>>,
	) -> Result<(), Vec<FailedToComputeCollider>> {
		Self::insert_collider_via::<AssetServer, Collider>(colliders, commands, server, meshes)
	}

	fn insert_collider_via<TAssetServer, TCollider>(
		mut colliders: Query<(Entity, &mut Self)>,
		mut commands: ZyheedaCommands,
		mut server: ResMut<TAssetServer>,
		meshes: Res<Assets<Mesh>>,
	) -> Result<(), Vec<FailedToComputeCollider>>
	where
		TAssetServer: Resource + LoadAsset,
		TCollider: Component + ConvexMeshCollider + ConcaveMeshCollider,
	{
		let mut errors = vec![];

		for (entity, mut async_collider) in &mut colliders {
			match async_collider.mesh.as_ref() {
				None => {
					async_collider.mesh = Some(server.load_asset(async_collider.path));
				}
				Some(mesh) => {
					let Some(mesh) = meshes.get(mesh) else {
						continue;
					};

					let Some(mut entity) = commands.get_mut(&entity) else {
						continue;
					};

					entity.try_remove::<Self>();

					let Some(collider) = async_collider.get_mesh_collider::<TCollider>(mesh) else {
						errors.push(FailedToComputeCollider {
							path: async_collider.path,
						});
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
pub(crate) struct FailedToComputeCollider {
	path: &'static str,
}

impl Display for FailedToComputeCollider {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}: cannot compute collider", self.path)
	}
}

impl ErrorData for FailedToComputeCollider {
	fn level(&self) -> Level {
		Level::Error
	}

	fn label() -> impl std::fmt::Display {
		"Failed to compute collider"
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

	impl AsyncCollider {
		fn with_mesh(mut self, mesh: Handle<Mesh>) -> Self {
			self.mesh = Some(mesh);
			self
		}
	}

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
	struct _Result(Result<(), Vec<FailedToComputeCollider>>);

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
	fn set_handle() {
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
			Some(&AsyncCollider::concave("my/path").with_mesh(handle)),
			app.world().entity(entity).get::<AsyncCollider>(),
		);
	}

	#[test]
	fn insert_convex_collider() {
		let handle = new_handle::<Mesh>();
		let mesh = Mesh::from(Cuboid::new(1., 2., 3.));
		let mut app = setup::<_Collider>(&[(handle.clone(), mesh)], MockAssetServer::default());
		let entity = app
			.world_mut()
			.spawn(AsyncCollider::convex("path").with_mesh(handle))
			.id();

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
		let entity = app
			.world_mut()
			.spawn(AsyncCollider::concave("path").with_mesh(handle))
			.id();

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
				AsyncCollider::concave("")
					.with_mesh(handle)
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
	#[test_case(AsyncCollider::concave("my/path").with_mesh(MESH_DOES_NOT_EXIST), true; "not when mesh missing")]
	#[test_case(AsyncCollider::concave("my/path").with_mesh(MESH_EXISTS), false; "when mesh exists")]
	fn remove_async_collider(collider: AsyncCollider, is_present: bool) {
		let mut app = setup::<_Collider>(
			&[(MESH_EXISTS, Mesh::from(Sphere::new(1.)))],
			MockAssetServer::default(),
		);
		let entity = app.world_mut().spawn(collider).id();

		app.update();

		assert_eq!(
			is_present,
			app.world().entity(entity).contains::<AsyncCollider>(),
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
	fn return_error_on_failure(collider_type: ColliderType) {
		let handle = new_handle();
		let mut app = setup::<_FaultyCollider>(
			&[(handle.clone(), Mesh::from(Sphere::new(1.)))],
			MockAssetServer::default(),
		);
		app.world_mut().spawn(AsyncCollider {
			collider_type,
			path: "my/path",
			mesh: Some(handle),
			scale: None,
		});

		app.update();

		assert_eq!(
			&_Result(Err(vec![FailedToComputeCollider { path: "my/path" }])),
			app.world().resource::<_Result>()
		);
	}

	#[test]
	fn remove_collider_on_failure() {
		let handle = new_handle();
		let mut app = setup::<_FaultyCollider>(
			&[(handle.clone(), Mesh::from(Sphere::new(1.)))],
			MockAssetServer::default(),
		);
		let entity = app
			.world_mut()
			.spawn(AsyncCollider {
				collider_type: ColliderType::Concave,
				path: "my/path",
				mesh: Some(handle),
				scale: None,
			})
			.id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<AsyncCollider>(),);
	}

	#[test]
	fn return_ok() {
		let handle = new_handle();
		let mut app = setup::<_Collider>(
			&[(handle.clone(), Mesh::from(Sphere::new(1.)))],
			MockAssetServer::default(),
		);
		app.world_mut()
			.spawn(AsyncCollider::concave("my/path").with_mesh(handle));

		app.update();

		assert_eq!(&_Result(Ok(())), app.world().resource::<_Result>());
	}
}
