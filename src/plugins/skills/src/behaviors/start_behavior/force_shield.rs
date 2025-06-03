use crate::behaviors::{SkillCaster, SkillTarget};
use bevy::ecs::system::EntityCommands;
use common::{
	effects::force_shield::ForceShield,
	traits::{handles_effect::HandlesEffect, handles_skill_behaviors::Spawner},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct StartForceShield;

impl StartForceShield {
	pub fn apply<TInteractions>(
		&self,
		entity: &mut EntityCommands,
		_: &SkillCaster,
		_: Spawner,
		_: &SkillTarget,
	) where
		TInteractions: HandlesEffect<ForceShield>,
	{
		entity.try_insert(TInteractions::effect(ForceShield));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::test_tools::utils::SingleThreadedApp;

	struct _HandlesInteractions;

	impl HandlesEffect<ForceShield> for _HandlesInteractions {
		type TTarget = ();
		type TEffectComponent = _ForceShield;

		fn effect(effect: ForceShield) -> Self::TEffectComponent {
			_ForceShield(effect)
		}

		fn attribute(_: Self::TTarget) -> impl Bundle {}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _ForceShield(ForceShield);

	fn force_shield(mut commands: Commands) -> Entity {
		let mut entity = commands.spawn_empty();
		StartForceShield.apply::<_HandlesInteractions>(
			&mut entity,
			&SkillCaster::from(Entity::from_raw(42)),
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

		let entity = app.world_mut().run_system_once(force_shield)?;

		assert_eq!(
			Some(&_ForceShield(ForceShield)),
			app.world().entity(entity).get::<_ForceShield>()
		);
		Ok(())
	}
}
