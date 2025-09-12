use crate::behaviors::{SkillCaster, SkillTarget};
use common::{
	effects::force::Force,
	traits::handles_physics::HandlesPhysicalEffect,
	zyheeda_commands::ZyheedaEntityCommands,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AttachForce;

impl AttachForce {
	pub fn attach<TPhysics>(
		&self,
		entity: &mut ZyheedaEntityCommands,
		_: &SkillCaster,
		_: &SkillTarget,
	) where
		TPhysics: HandlesPhysicalEffect<Force>,
	{
		entity.try_insert(TPhysics::into_effect_component(Force));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::{
		attributes::effect_target::EffectTarget,
		components::persistent_entity::PersistentEntity,
	};
	use std::sync::LazyLock;
	use testing::SingleThreadedApp;

	struct _HandlesInteractions;

	impl HandlesPhysicalEffect<Force> for _HandlesInteractions {
		type TEffectComponent = _Force;
		type TAffectedComponent = _Affected;

		fn into_effect_component(effect: Force) -> _Force {
			_Force(effect)
		}

		fn into_affected_component(_: EffectTarget<Force>) -> _Affected {
			_Affected
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Force(Force);

	#[derive(Component)]
	struct _Affected;

	static CASTER: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	fn force(mut commands: Commands) -> Entity {
		let mut entity = commands.spawn(()).into();
		AttachForce.attach::<_HandlesInteractions>(
			&mut entity,
			&SkillCaster::from(*CASTER),
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
