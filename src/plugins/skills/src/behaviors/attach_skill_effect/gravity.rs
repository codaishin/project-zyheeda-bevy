use crate::behaviors::{SkillCaster, SkillTarget};
use common::{
	effects::gravity::Gravity,
	tools::UnitsPerSecond,
	traits::handles_effect::HandlesEffect,
	zyheeda_commands::ZyheedaEntityCommands,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AttachGravity {
	strength: UnitsPerSecond,
}

impl AttachGravity {
	pub fn attach<TInteractions>(
		&self,
		entity: &mut ZyheedaEntityCommands,
		_: &SkillCaster,
		_: &SkillTarget,
	) where
		TInteractions: HandlesEffect<Gravity>,
	{
		entity.try_insert(TInteractions::effect(Gravity {
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
		components::persistent_entity::PersistentEntity,
		traits::clamp_zero_positive::ClampZeroPositive,
		zyheeda_commands::ZyheedaCommands,
	};
	use std::sync::LazyLock;
	use testing::SingleThreadedApp;

	struct _HandlesEffects;

	impl HandlesEffect<Gravity> for _HandlesEffects {
		type TTarget = ();
		type TEffectComponent = _Effect;

		fn effect(effect: Gravity) -> _Effect {
			_Effect(effect)
		}

		fn attribute(_: Self::TTarget) -> impl Bundle {}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Effect(Gravity);

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
			.run_system_once_with(gravity, UnitsPerSecond::new(42.))?;

		assert_eq!(
			Some(&_Effect(Gravity {
				strength: UnitsPerSecond::new(42.),
			})),
			app.world().entity(entity).get::<_Effect>()
		);
		Ok(())
	}
}
