use crate::behaviors::{SkillCaster, SkillSpawner, SkillTarget};
use bevy::ecs::system::EntityCommands;
use common::{effects::deal_damage::DealDamage, traits::handles_effect::HandlesEffect};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum StartDealingDamage {
	SingleTarget(f32),
	Piercing(f32),
	OverTime(f32),
}

impl StartDealingDamage {
	pub fn apply<TEffects>(
		&self,
		entity: &mut EntityCommands,
		_: &SkillCaster,
		_: &SkillSpawner,
		_: &SkillTarget,
	) where
		TEffects: HandlesEffect<DealDamage>,
	{
		entity.try_insert(TEffects::effect(match *self {
			Self::SingleTarget(dmg) => DealDamage::once(dmg),
			Self::Piercing(dmg) => DealDamage::once_per_target(dmg),
			Self::OverTime(dmg) => DealDamage::once_per_second(dmg),
		}));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{ecs::system::RunSystemOnce, prelude::*};
	use common::test_tools::utils::SingleThreadedApp;

	struct _HandlesDamage;

	impl HandlesEffect<DealDamage> for _HandlesDamage {
		type TTarget = ();
		type TEffectComponent = _Effect;

		fn effect(effect: DealDamage) -> _Effect {
			_Effect(effect)
		}

		fn attribute(_: Self::TTarget) -> impl Bundle {}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Effect(DealDamage);

	struct _HandlesShading;

	fn damage(damage: StartDealingDamage) -> impl Fn(Commands) -> Entity {
		move |mut commands| {
			let mut entity = commands.spawn_empty();
			damage.apply::<_HandlesDamage>(
				&mut entity,
				&SkillCaster::from(Entity::from_raw(42)),
				&SkillSpawner::from(Entity::from_raw(43)),
				&SkillTarget::default(),
			);
			entity.id()
		}
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_single_target_damage() {
		let mut app = setup();

		let start_dealing_damage = StartDealingDamage::SingleTarget(42.);
		let entity = app
			.world_mut()
			.run_system_once(damage(start_dealing_damage));

		assert_eq!(
			Some(&_Effect(DealDamage::once(42.))),
			app.world().entity(entity).get::<_Effect>(),
		);
	}

	#[test]
	fn insert_piercing_damage() {
		let mut app = setup();

		let start_dealing_damage = StartDealingDamage::Piercing(42.);
		let entity = app
			.world_mut()
			.run_system_once(damage(start_dealing_damage));

		assert_eq!(
			Some(&_Effect(DealDamage::once_per_target(42.))),
			app.world().entity(entity).get::<_Effect>(),
		);
	}

	#[test]
	fn insert_over_time_damage() {
		let mut app = setup();

		let start_dealing_damage = StartDealingDamage::OverTime(42.);
		let entity = app
			.world_mut()
			.run_system_once(damage(start_dealing_damage));

		assert_eq!(
			Some(&_Effect(DealDamage::once_per_second(42.))),
			app.world().entity(entity).get::<_Effect>(),
		);
	}
}
