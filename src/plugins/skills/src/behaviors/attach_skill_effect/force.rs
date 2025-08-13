use crate::behaviors::{SkillCaster, SkillTarget};
use common::{
	effects::force::Force,
	traits::handles_effect::HandlesEffect,
	zyheeda_commands::ZyheedaEntityCommands,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct AttachForce;

impl AttachForce {
	pub fn attach<TInteractions>(
		&self,
		entity: &mut ZyheedaEntityCommands,
		_: &SkillCaster,
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
