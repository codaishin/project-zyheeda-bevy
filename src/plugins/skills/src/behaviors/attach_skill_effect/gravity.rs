use crate::behaviors::{SkillCaster, SkillTarget};
use common::{
	effects::gravity::Gravity,
	tools::UnitsPerSecond,
	traits::handles_physics::HandlesPhysicalEffect,
	zyheeda_commands::ZyheedaEntityCommands,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AttachGravity {
	strength: UnitsPerSecond,
}

impl AttachGravity {
	pub fn attach<TPhysics>(
		&self,
		entity: &mut ZyheedaEntityCommands,
		_: &SkillCaster,
		_: &SkillTarget,
	) where
		TPhysics: HandlesPhysicalEffect<Gravity>,
	{
		entity.try_insert(TPhysics::into_effect_component(Gravity {
			strength: self.strength,
		}));
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
		zyheeda_commands::ZyheedaCommands,
	};
	use std::sync::LazyLock;
	use testing::SingleThreadedApp;

	struct _HandlesEffects;

	impl HandlesPhysicalEffect<Gravity> for _HandlesEffects {
		type TEffectComponent = _Effect;
		type TAffectedComponent = _Affected;

		fn into_effect_component(effect: Gravity) -> _Effect {
			_Effect(effect)
		}

		fn into_affected_component(_: EffectTarget<Gravity>) -> _Affected {
			_Affected
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Effect(Gravity);

	#[derive(Component)]
	struct _Affected;

	static CASTER: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	fn gravity(In(pull): In<UnitsPerSecond>, mut commands: ZyheedaCommands) -> Entity {
		let mut entity = commands.spawn(()).into();
		AttachGravity { strength: pull }.attach::<_HandlesEffects>(
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
	fn spawn_gravity_effect() -> Result<(), RunSystemError> {
		let mut app = setup();

		let entity = app
			.world_mut()
			.run_system_once_with(gravity, UnitsPerSecond::from(42.))?;

		assert_eq!(
			Some(&_Effect(Gravity {
				strength: UnitsPerSecond::from(42.),
			})),
			app.world().entity(entity).get::<_Effect>()
		);
		Ok(())
	}
}
