use crate::components::enemy::{Enemy, attack_phase::EnemyAttackPhase};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::traits::{
	accessors::get::GetContextMut,
	handles_skills_control::{HoldSkill, SkillControl},
};

impl Enemy {
	pub(crate) fn hold_attack<TSkills>(
		mut skills: StaticSystemParam<TSkills>,
		enemies: Query<(Entity, &EnemyAttackPhase)>,
	) where
		TSkills: for<'c> GetContextMut<SkillControl, TContext<'c>: HoldSkill>,
	{
		for (entity, phase) in &enemies {
			let EnemyAttackPhase::HoldSkill { key, .. } = phase else {
				continue;
			};

			let ctx = TSkills::get_context_mut(&mut skills, SkillControl { entity });
			let Some(mut ctx) = ctx else {
				continue;
			};

			ctx.holding(*key);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::enemy::attack_phase::EnemyAttackPhase;
	use common::tools::action_key::slot::SlotKey;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::time::Duration;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component, NestedMocks)]
	struct _Skills {
		mock: Mock_Skills,
	}

	#[automock]
	impl HoldSkill for _Skills {
		fn holding<TSlot>(&mut self, key: TSlot)
		where
			TSlot: Into<SlotKey> + 'static,
		{
			self.mock.holding(key);
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, Enemy::hold_attack::<Query<&mut _Skills>>);

		app
	}

	#[test]
	fn holding_skills() {
		let mut app = setup();
		app.world_mut().spawn((
			EnemyAttackPhase::HoldSkill {
				key: SlotKey(42),
				holding: Duration::default(),
			},
			_Skills::new().with_mock(|mock| {
				mock.expect_holding()
					.once()
					.with(eq(SlotKey(42)))
					.return_const(());
			}),
		));

		app.update();
	}
}
