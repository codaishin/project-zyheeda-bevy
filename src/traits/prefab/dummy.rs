use super::{simple_collidable::SimpleCollidablePrefab, AssetKey, CreatePrefab, Instantiate};
use crate::{
	bundles::ColliderBundle,
	components::{ColliderRoot, Dummy},
	errors::Error,
	resources::Prefab,
};
use bevy::{
	asset::{Assets, Handle},
	ecs::system::ResMut,
	hierarchy::BuildChildren,
	math::Vec3,
	pbr::{PbrBundle, StandardMaterial},
	render::{
		color::Color,
		mesh::{shape, Mesh},
	},
	transform::components::Transform,
	utils::default,
};
use bevy_rapier3d::geometry::Collider;

pub type DummyPrefab = SimpleCollidablePrefab<Dummy, (), PbrBundle>;

const DUMMY_DIMENSIONS: Vec3 = Vec3 {
	x: 0.4,
	y: 2.,
	z: 0.4,
};

impl CreatePrefab<DummyPrefab, StandardMaterial> for Dummy {
	fn create_prefab(
		mut materials: ResMut<Assets<StandardMaterial>>,
		mut meshes: ResMut<Assets<Mesh>>,
	) -> Result<DummyPrefab, Error> {
		let transform = Transform::from_xyz(0., 1., 0.);

		Ok(Prefab::new(
			(),
			(
				[PbrBundle {
					transform,
					material: materials.add(StandardMaterial {
						base_color: Color::GRAY,
						..default()
					}),
					mesh: meshes.add(Mesh::from(shape::Box::new(
						DUMMY_DIMENSIONS.x,
						DUMMY_DIMENSIONS.y,
						DUMMY_DIMENSIONS.z,
					))),
					..default()
				}],
				ColliderBundle::new_static_collider(
					transform,
					Collider::cuboid(
						DUMMY_DIMENSIONS.x / 2.,
						DUMMY_DIMENSIONS.y / 2.,
						DUMMY_DIMENSIONS.z / 2.,
					),
				),
			),
		))
	}
}

impl Instantiate for Dummy {
	fn instantiate(
		on: &mut bevy::ecs::system::EntityCommands,
		mut get_mesh_handle: impl FnMut(AssetKey, Mesh) -> Handle<Mesh>,
		mut get_material_handle: impl FnMut(AssetKey, StandardMaterial) -> Handle<StandardMaterial>,
	) -> Result<(), Error> {
		let transform = Transform::from_xyz(0., 1., 0.);
		let key = AssetKey::Dummy;
		let mesh = Mesh::from(shape::Box::new(
			DUMMY_DIMENSIONS.x,
			DUMMY_DIMENSIONS.y,
			DUMMY_DIMENSIONS.z,
		));
		let material = StandardMaterial {
			base_color: Color::GRAY,
			..default()
		};
		let collider = Collider::cuboid(
			DUMMY_DIMENSIONS.x / 2.,
			DUMMY_DIMENSIONS.y / 2.,
			DUMMY_DIMENSIONS.z / 2.,
		);

		on.with_children(|parent| {
			parent.spawn(PbrBundle {
				transform,
				mesh: get_mesh_handle(key, mesh),
				material: get_material_handle(key, material),
				..default()
			});
			parent.spawn((
				ColliderBundle::new_static_collider(transform, collider),
				ColliderRoot(parent.parent_entity()),
			));
		});
		Ok(())
	}
}
