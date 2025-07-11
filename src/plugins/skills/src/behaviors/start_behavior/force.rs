use crate::behaviors::{SkillCaster, SkillTarget};
use bevy::ecs::system::EntityCommands;
use common::{
	effects::force::Force,
	traits::{handles_effect::HandlesEffect, handles_skill_behaviors::Spawner},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct StartForce;

impl StartForce {
	pub fn apply<TInteractions>(
		&self,
		entity: &mut EntityCommands,
		_: &SkillCaster,
		_: Spawner,
		_: &SkillTarget,
	) where
		TInteractions: HandlesEffect<Force>,
	{
		entity.try_insert(TInteractions::effect(Force));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::components::persistent_entity::PersistentEntity;
	use std::sync::LazyLock;
	use testing::SingleThreadedApp;

	struct _HandlesInteractions;

	impl HandlesEffect<Force> for _HandlesInteractions {
		type TTarget = ();
		type TEffectComponent = _Force;

		fn effect(effect: Force) -> Self::TEffectComponent {
			_Force(effect)
		}

		fn attribute(_: Self::TTarget) -> impl Bundle {}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Force(Force);

	static CASTER: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	fn force(mut commands: Commands) -> Entity {
		let mut entity = commands.spawn_empty();
		StartForce.apply::<_HandlesInteractions>(
			&mut entity,
			&SkillCaster::from(*CASTER),
			Spawner::Center,
			&SkillTarget::default(),
		);
		entity.id()
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn spawn_force_marker() -> Result<(), RunSystemError> {
		let mut app = setup();

		let entity = app.world_mut().run_system_once(force)?;

		assert_eq!(
			Some(&_Force(Force)),
			app.world().entity(entity).get::<_Force>()
		);
		Ok(())
	}
}
