use bevy::{
	ecs::system::EntityCommands,
	hierarchy::BuildChildren,
	math::Vec3,
	prelude::*,
	utils::default,
};
use bevy_rapier3d::{
	dynamics::RigidBody,
	geometry::Collider,
	prelude::{ActiveCollisionTypes, ActiveEvents, Sensor},
};
use common::{
	bundles::{AssetModelBundle, ColliderBundle, ColliderTransformBundle},
	components::{AssetModel, ColliderRoot},
	errors::Error,
	traits::{
		handles_effect_shading::HandlesEffectShading,
		prefab::{GetOrCreateAssets, Prefab},
	},
};

#[derive(Component, Debug, PartialEq)]
pub struct ShieldContact {
	pub location: Entity,
}

impl<TShadersPlugin> Prefab<TShadersPlugin> for ShieldContact
where
	TShadersPlugin: HandlesEffectShading,
{
	fn instantiate_on<TAfterInstantiation>(
		&self,
		entity: &mut EntityCommands,
		_: impl GetOrCreateAssets,
	) -> Result<(), Error> {
		let half_size = Vec3 {
			x: 0.5,
			y: 0.5,
			z: 0.05,
		};
		let model = AssetModel::path("models/shield.glb");

		entity
			.insert((RigidBody::Fixed, TShadersPlugin::effect_shader_target()))
			.with_children(|parent| {
				parent.spawn(AssetModelBundle { model, ..default() });
				parent.spawn((
					ColliderTransformBundle {
						collider: Collider::cuboid(half_size.x, half_size.y, half_size.z),
						active_events: ActiveEvents::COLLISION_EVENTS,
						active_collision_types: ActiveCollisionTypes::STATIC_STATIC,
						..default()
					},
					Sensor,
					ColliderRoot(parent.parent_entity()),
				));
			});

		Ok(())
	}
}

#[derive(Component, Debug, PartialEq)]
pub struct ShieldProjection;

impl<TShadersPlugin> Prefab<TShadersPlugin> for ShieldProjection
where
	TShadersPlugin: HandlesEffectShading,
{
	fn instantiate_on<TAfterInstantiation>(
		&self,
		entity: &mut EntityCommands,
		_: impl GetOrCreateAssets,
	) -> Result<(), Error> {
		let radius = 1.;
		let transform = Transform::from_xyz(0., 0., -radius).with_scale(Vec3::splat(radius * 2.));

		entity.try_insert((
			TShadersPlugin::effect_shader_target(),
			AssetModelBundle {
				model: AssetModel::path("models/sphere.glb"),
				transform,
				..default()
			},
			ColliderBundle {
				collider: Collider::ball(0.5),
				active_events: ActiveEvents::COLLISION_EVENTS,
				active_collision_types: ActiveCollisionTypes::STATIC_STATIC,
			},
			Sensor,
		));

		Ok(())
	}
}
