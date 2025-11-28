use crate::{
	components::queue::Queue,
	system_parameters::loadout_activity::LoadoutActivityContext,
};
use common::traits::{
	handles_loadout::{ActiveSkill, ActiveSkills},
	iterate::Iterate,
};

impl ActiveSkills for LoadoutActivityContext<'_> {
	type TIter<'a>
		= ActiveSkillsIter<'a>
	where
		Self: 'a;

	fn active_skills(&self) -> Self::TIter<'_> {
		let queue = Some(self.queue.as_ref().iterate());

		ActiveSkillsIter { queue }
	}
}

pub struct ActiveSkillsIter<'a> {
	queue: Option<<Queue as Iterate<'a>>::TIter>,
}

impl Iterator for ActiveSkillsIter<'_> {
	type Item = ActiveSkill;

	fn next(&mut self) -> Option<Self::Item> {
		let mut queue = self.queue.take()?;
		let queued = queue.next()?;

		Some(ActiveSkill {
			key: queued.key,
			animate: queued.skill.animate,
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::queue::Queue,
		skills::{QueuedSkill, Skill},
		system_parameters::loadout_activity::LoadoutActivity,
	};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::{
		tools::action_key::slot::SlotKey,
		traits::{accessors::get::GetContext, handles_loadout::skills::Skills},
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn return_only_queued_skill_as_active() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Queue::from([QueuedSkill::new(
				Skill::default(),
				SlotKey(42),
			)]))
			.id();

		let active_skills = app.world_mut().run_system_once(move |p: LoadoutActivity| {
			let ctx = LoadoutActivity::get_context(&p, Skills { entity }).unwrap();
			ctx.active_skills().collect::<Vec<_>>()
		})?;

		assert_eq!(
			vec![ActiveSkill {
				key: SlotKey(42),
				animate: false,
			}],
			active_skills
		);
		Ok(())
	}

	#[test]
	fn return_only_first_queued_skill_as_active() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Queue::from([
				QueuedSkill::new(Skill::default(), SlotKey(42)),
				QueuedSkill::new(Skill::default(), SlotKey(11)),
			]))
			.id();

		let active_skills = app.world_mut().run_system_once(move |p: LoadoutActivity| {
			let ctx = LoadoutActivity::get_context(&p, Skills { entity }).unwrap();
			ctx.active_skills().collect::<Vec<_>>()
		})?;

		assert_eq!(
			vec![ActiveSkill {
				key: SlotKey(42),
				animate: false,
			}],
			active_skills
		);
		Ok(())
	}

	#[test]
	fn return_animated_true() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(Queue::from([QueuedSkill::new(
				Skill {
					animate: true,
					..default()
				},
				SlotKey(42),
			)]))
			.id();

		let active_skills = app.world_mut().run_system_once(move |p: LoadoutActivity| {
			let ctx = LoadoutActivity::get_context(&p, Skills { entity }).unwrap();
			ctx.active_skills().collect::<Vec<_>>()
		})?;

		assert_eq!(
			vec![ActiveSkill {
				key: SlotKey(42),
				animate: true,
			}],
			active_skills
		);
		Ok(())
	}
}
