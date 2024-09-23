use bevy::{
	ecs::system::EntityCommands,
	hierarchy::BuildChildren,
	math::Vec3,
	prelude::{Component, Entity, Transform},
	utils::default,
};
use bevy_rapier3d::{dynamics::RigidBody, geometry::Collider, prelude::Sensor};
use common::{
	bundles::{AssetModelBundle, ColliderTransformBundle},
	components::{AssetModel, ColliderRoot},
	errors::Error,
};
use prefabs::traits::{GetOrCreateAssets, Instantiate};

#[derive(Component, Debug, PartialEq)]
pub struct ShieldContact {
	pub location: Entity,
}

impl Instantiate for ShieldContact {
	fn instantiate(&self, on: &mut EntityCommands, _: impl GetOrCreateAssets) -> Result<(), Error> {
		let half_size = Vec3 {
			x: 0.5,
			y: 0.5,
			z: 0.05,
		};
		let model = AssetModel("models/shield.glb#Scene0");

		on.insert(RigidBody::Fixed).with_children(|parent| {
			parent.spawn(AssetModelBundle { model, ..default() });
			parent.spawn((
				ColliderTransformBundle::new_static_collider(
					default(),
					Collider::cuboid(half_size.x, half_size.y, half_size.z),
				),
				Sensor,
				ColliderRoot(parent.parent_entity()),
			));
		});

		Ok(())
	}
}

#[derive(Component, Debug, PartialEq)]
pub struct ShieldProjection;

impl Instantiate for ShieldProjection {
	fn instantiate(&self, on: &mut EntityCommands, _: impl GetOrCreateAssets) -> Result<(), Error> {
		let half_size = Vec3 {
			x: 0.5,
			y: 0.5,
			z: 0.5,
		};
		on.try_insert((
			ColliderTransformBundle::new_static_collider(
				Transform::from_xyz(0., 0., -half_size.z),
				Collider::cuboid(half_size.x, half_size.y, half_size.z),
			),
			Sensor,
		));

		Ok(())
	}
}
