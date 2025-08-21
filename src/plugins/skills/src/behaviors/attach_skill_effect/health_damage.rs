use crate::behaviors::{SkillCaster, SkillTarget};
use common::{
	effects::health_damage::HealthDamage,
	traits::handles_effects::HandlesEffect,
	zyheeda_commands::ZyheedaEntityCommands,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum AttachHealthDamage {
	OneTime(f32),
	OverTime(f32),
}

impl AttachHealthDamage {
	pub fn attach<TInteractions>(
		&self,
		entity: &mut ZyheedaEntityCommands,
		_: &SkillCaster,
		_: &SkillTarget,
	) where
		TInteractions: HandlesEffect<HealthDamage>,
	{
		entity.try_insert(TInteractions::effect(match *self {
			Self::OneTime(dmg) => HealthDamage::once(dmg),
			Self::OverTime(dmg) => HealthDamage::once_per_second(dmg),
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
	use common::{attributes::health::Health, components::persistent_entity::PersistentEntity};
	use std::sync::LazyLock;
	use testing::SingleThreadedApp;

	struct _HandlesDamage;

	impl HandlesEffect<HealthDamage> for _HandlesDamage {
		type TEffectComponent = _Effect;

		fn effect(effect: HealthDamage) -> _Effect {
			_Effect(effect)
		}

		fn attribute(_: Health) -> impl Bundle {}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Effect(HealthDamage);

	struct _HandlesShading;

	static CASTER: LazyLock<PersistentEntity> = LazyLock::new(PersistentEntity::default);

	fn damage(damage: AttachHealthDamage) -> impl Fn(Commands) -> Entity {
		move |mut commands| {
			let mut entity = commands.spawn(()).into();
			damage.attach::<_HandlesDamage>(
				&mut entity,
				&SkillCaster::from(*CASTER),
				&SkillTarget::default(),
			);
			entity.id()
		}
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_single_target_damage() -> Result<(), RunSystemError> {
		let mut app = setup();

		let start_dealing_damage = AttachHealthDamage::OneTime(42.);
		let entity = app
			.world_mut()
			.run_system_once(damage(start_dealing_damage))?;

		assert_eq!(
			Some(&_Effect(HealthDamage::once(42.))),
			app.world().entity(entity).get::<_Effect>(),
		);
		Ok(())
	}

	#[test]
	fn insert_over_time_damage() -> Result<(), RunSystemError> {
		let mut app = setup();

		let start_dealing_damage = AttachHealthDamage::OverTime(42.);
		let entity = app
			.world_mut()
			.run_system_once(damage(start_dealing_damage))?;

		assert_eq!(
			Some(&_Effect(HealthDamage::once_per_second(42.))),
			app.world().entity(entity).get::<_Effect>(),
		);
		Ok(())
	}
}
