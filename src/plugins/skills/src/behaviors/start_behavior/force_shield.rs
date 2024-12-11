use crate::behaviors::{SkillCaster, SkillSpawner, SkillTarget};
use bevy::ecs::system::EntityCommands;
use common::{
	blocker::Blocker,
	effects::force_shield::ForceShield,
	traits::handles_effect_shading::HandlesEffectShadingFor,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct StartForceShield;

impl StartForceShield {
	pub fn apply<TShaders>(
		&self,
		entity: &mut EntityCommands,
		_: &SkillCaster,
		_: &SkillSpawner,
		_: &SkillTarget,
	) where
		TShaders: HandlesEffectShadingFor<ForceShield>,
	{
		entity.try_insert((
			Blocker::insert([Blocker::Force]),
			TShaders::effect_shader(ForceShield),
		));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{ecs::system::RunSystemOnce, prelude::*};
	use common::{blocker::BlockerInsertCommand, test_tools::utils::SingleThreadedApp};

	struct _HandlesShading;

	impl HandlesEffectShadingFor<ForceShield> for _HandlesShading {
		fn effect_shader(effect: ForceShield) -> impl Bundle {
			_ForceShield(effect)
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _ForceShield(ForceShield);

	fn force_shield(mut commands: Commands) -> Entity {
		let mut entity = commands.spawn_empty();
		StartForceShield.apply::<_HandlesShading>(
			&mut entity,
			&SkillCaster::from(Entity::from_raw(42)),
			&SkillSpawner::from(Entity::from_raw(43)),
			&SkillTarget::default(),
		);
		entity.id()
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn spawn_force() {
		let mut app = setup();

		let entity = app.world_mut().run_system_once(force_shield);

		assert_eq!(
			Some(&Blocker::insert([Blocker::Force])),
			app.world().entity(entity).get::<BlockerInsertCommand>()
		);
	}

	#[test]
	fn spawn_force_marker() {
		let mut app = setup();

		let entity = app.world_mut().run_system_once(force_shield);

		assert_eq!(
			Some(&_ForceShield(ForceShield)),
			app.world().entity(entity).get::<_ForceShield>()
		);
	}
}
