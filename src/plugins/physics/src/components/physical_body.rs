use bevy::{
	ecs::{query::QueryEntityError, system::StaticSystemParam},
	prelude::*,
};
use bevy_rapier3d::prelude::*;
use common::traits::{
	handles_physics::physical_bodies::{Body, PhysicsType},
	prefab::{Prefab, PrefabEntityCommands},
};

#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
pub struct PhysicalBody(pub(crate) Body);

impl PhysicalBody {
	const COLLISION_GROUP: Group = Group::GROUP_1;
	const TERRAIN_GROUP: Group = Group::GROUP_2;
}

impl From<Body> for PhysicalBody {
	fn from(body: Body) -> Self {
		Self(body)
	}
}

impl Prefab<()> for PhysicalBody {
	type TError = QueryEntityError;
	type TSystemParam<'w, 's> = Query<'w, 's, &'static PhysicalBody>;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		bodies: StaticSystemParam<Self::TSystemParam<'_, '_>>,
	) -> Result<(), Self::TError> {
		let body = bodies.get(entity.entity_id())?;
		let groups = match body.0.physics_type {
			PhysicsType::Agent => Self::COLLISION_GROUP,
			PhysicsType::Terrain => Self::COLLISION_GROUP | Self::TERRAIN_GROUP,
		};

		entity.try_insert(CollisionGroups::new(groups, Self::COLLISION_GROUP));

		Ok(())
	}
}
