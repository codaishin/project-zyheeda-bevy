mod contact;
mod dto;
mod motion;
mod projection;

use crate::components::{
	effect::{force::ForceEffect, gravity::GravityEffect, health_damage::HealthDamageEffect},
	skill::dto::SkillDto,
};
use bevy::prelude::*;
use common::{
	components::{
		asset_model::AssetModel,
		insert_asset::InsertAsset,
		persistent_entity::PersistentEntity,
	},
	tools::Units,
	traits::handles_skill_physics::{Contact, Effect, Projection},
	zyheeda_commands::ZyheedaEntityCommands,
};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};
use std::{sync::LazyLock, time::Duration};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone)]
#[require(PersistentEntity)]
#[savable_component(dto = SkillDto)]
pub struct Skill {
	pub(crate) lifetime: Option<Duration>,
	pub(crate) created_from: CreatedFrom,
	pub(crate) contact: Contact,
	pub(crate) contact_effects: Vec<Effect>,
	pub(crate) projection: Projection,
	pub(crate) projection_effects: Vec<Effect>,
}

#[derive(Component, Debug, PartialEq)]
pub struct SkillContact;

#[derive(Component, Debug, PartialEq)]
pub struct SkillProjection;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum CreatedFrom {
	Spawn,
	Save,
}

const SPHERE_MODEL: &str = "models/sphere.glb";
const BEAM_MODEL: fn() -> Mesh = || {
	Mesh::from(Cylinder {
		radius: 1.,
		half_height: 0.5,
	})
};
const HALF_FORWARD: Transform = Transform::from_translation(Vec3 {
	x: 0.,
	y: 0.,
	z: -0.5,
});
static HOLLOW_OUTER_THICKNESS: LazyLock<Units> = LazyLock::new(|| Units::from(0.3));

struct Beam;

enum Model {
	Asset(AssetModel),
	Proc(InsertAsset<Mesh>),
}

fn insert_effect(entity: &mut ZyheedaEntityCommands, effect: Effect) {
	match effect {
		Effect::Force(force) => entity.try_insert(ForceEffect(force)),
		Effect::Gravity(gravity) => entity.try_insert(GravityEffect(gravity)),
		Effect::HealthDamage(health_damage) => entity.try_insert(HealthDamageEffect(health_damage)),
	};
}

#[cfg(test)]
mod test_impls {
	use super::*;
	use common::{
		tools::Units,
		traits::handles_skill_physics::{
			ContactShape,
			Motion,
			ProjectionShape,
			SkillCaster,
			SkillTarget,
		},
	};
	use std::collections::HashSet;

	impl Default for Skill {
		fn default() -> Self {
			Self {
				lifetime: None,
				created_from: CreatedFrom::Spawn,
				contact: Contact {
					shape: ContactShape::Beam {
						range: Units::from_u8(10),
						radius: Units::from_u8(1),
						blocked_by: HashSet::from([]),
					},
					motion: Motion::Stationary {
						caster: SkillCaster(PersistentEntity::default()),
						max_cast_range: Units::from_u8(1),
						target: SkillTarget::Ground(Vec3::ZERO),
					},
				},
				contact_effects: vec![],
				projection: Projection {
					shape: ProjectionShape::Beam {
						radius: Units::from_u8(2),
					},
					offset: None,
				},
				projection_effects: vec![],
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		effects::{
			EffectApplies::Once,
			force::Force,
			gravity::Gravity,
			health_damage::HealthDamage,
		},
		tools::UnitsPerSecond,
		traits::accessors::get::TryApplyOn,
		zyheeda_commands::ZyheedaCommands,
	};
	use std::fmt::Debug;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	const FORCE: Force = Force;
	const HEALTH_DAMAGE: HealthDamage = HealthDamage(11., Once);
	const GRAVITY: Gravity = Gravity {
		strength: UnitsPerSecond::from_u8(11),
	};

	#[test_case(Effect::Force(FORCE), ForceEffect(FORCE))]
	#[test_case(Effect::HealthDamage(HEALTH_DAMAGE), HealthDamageEffect(HEALTH_DAMAGE))]
	#[test_case(Effect::Gravity(GRAVITY), GravityEffect(GRAVITY))]
	fn do_insert_effect<T>(effect: Effect, component: T) -> Result<(), RunSystemError>
	where
		T: Component + Debug + PartialEq,
	{
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut c: ZyheedaCommands| {
				c.try_apply_on(&entity, |mut e| {
					insert_effect(&mut e, effect);
				});
			})?;

		assert_eq!(Some(&component), app.world().entity(entity).get::<T>());
		Ok(())
	}
}
