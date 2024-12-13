use bevy::{ecs::system::EntityCommands, math::Vec3, prelude::*, utils::default};
use bevy_rapier3d::{
	geometry::Collider,
	prelude::{ActiveCollisionTypes, ActiveEvents, Sensor},
};
use common::{
	bundles::{AssetModelBundle, ColliderBundle},
	components::AssetModel,
	errors::Error,
	traits::{
		handles_effect_shading::HandlesEffectShading,
		prefab::{GetOrCreateAssets, Prefab},
	},
};

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
