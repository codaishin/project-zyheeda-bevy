use super::{GetOrCreateAssets, Instantiate};
use bevy::{
	ecs::system::EntityCommands,
	hierarchy::BuildChildren,
	math::{primitives::Cuboid, Vec3},
	pbr::{PbrBundle, StandardMaterial},
	render::{color::Color, mesh::Mesh},
	transform::components::Transform,
	utils::default,
};
use bevy_rapier3d::geometry::Collider;
use common::{
	bundles::ColliderTransformBundle,
	components::{ColliderRoot, Dummy},
	errors::Error,
	traits::cache::GetOrCreateTypeAsset,
};

const DUMMY_DIMENSIONS: Vec3 = Vec3 {
	x: 0.4,
	y: 2.,
	z: 0.4,
};

impl Instantiate for Dummy {
	fn instantiate(
		&self,
		on: &mut EntityCommands,
		mut assets: impl GetOrCreateAssets,
	) -> Result<(), Error> {
		let transform = Transform::from_xyz(0., 1., 0.);
		let mesh = assets.get_or_create_for::<Dummy>(|| {
			Mesh::from(Cuboid::new(
				DUMMY_DIMENSIONS.x,
				DUMMY_DIMENSIONS.y,
				DUMMY_DIMENSIONS.z,
			))
		});
		let material = assets.get_or_create_for::<Dummy>(|| StandardMaterial {
			base_color: Color::GRAY,
			..default()
		});
		let collider = Collider::cuboid(
			DUMMY_DIMENSIONS.x / 2.,
			DUMMY_DIMENSIONS.y / 2.,
			DUMMY_DIMENSIONS.z / 2.,
		);

		on.with_children(|parent| {
			parent.spawn(PbrBundle {
				transform,
				mesh,
				material,
				..default()
			});
			parent.spawn((
				ColliderTransformBundle::new_static_collider(transform, collider),
				ColliderRoot(parent.parent_entity()),
			));
		});
		Ok(())
	}
}
