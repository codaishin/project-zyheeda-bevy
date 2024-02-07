use super::{simple_model::SimpleModelPrefab, CreatePrefab};
use crate::{bundles::ColliderBundle, components::Dummy, errors::Error, resources::Prefab};
use bevy::{
	asset::Assets,
	ecs::system::ResMut,
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

const DUMMY_DIMENSIONS: Vec3 = Vec3 {
	x: 0.4,
	y: 2.,
	z: 0.4,
};

impl CreatePrefab<SimpleModelPrefab<Dummy, ()>, StandardMaterial> for Dummy {
	fn create_prefab(
		mut materials: ResMut<Assets<StandardMaterial>>,
		mut meshes: ResMut<Assets<Mesh>>,
	) -> Result<SimpleModelPrefab<Dummy, ()>, Error> {
		let transform = Transform::from_xyz(0., 1., 0.);

		Ok(Prefab::new(
			(),
			(
				PbrBundle {
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
				},
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
