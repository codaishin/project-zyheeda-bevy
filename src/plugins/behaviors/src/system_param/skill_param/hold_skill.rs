use crate::{components::skill_usage::SkillUsage, system_param::skill_param::SkillContextMut};
use common::{tools::action_key::slot::SlotKey, traits::handles_skills_control::HoldSkill};

impl HoldSkill for SkillContextMut<'_> {
	fn holding<TSlot>(&mut self, key: TSlot)
	where
		TSlot: Into<SlotKey>,
	{
		let key = key.into();

		match self {
			SkillContextMut::Mut(usage) => usage.holding_internal(key),
			SkillContextMut::New { usage, .. } => usage.holding_internal(key),
		}
	}
}

impl SkillUsage {
	fn holding_internal(&mut self, key: SlotKey) {
		if !self.started_holding.remove(&key) && !self.holding.contains(&key) {
			self.started_holding.insert(key);
		}

		self.holding.insert(key);
		self.refreshed.insert(key);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::skill_usage::SkillUsage, system_param::skill_param::SkillParamMut};
	use bevy::{
		app::{App, Update},
		ecs::system::{RunSystemError, RunSystemOnce},
	};
	use common::traits::{accessors::get::GetContextMut, handles_skills_control::SkillControl};
	use std::collections::HashSet;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn start_holding() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(SkillUsage::default()).id();

		app.world_mut()
			.run_system_once(move |mut p: SkillParamMut| {
				let mut ctx =
					SkillParamMut::get_context_mut(&mut p, SkillControl { entity }).unwrap();
				ctx.holding(SlotKey(42));
			})?;

		assert_eq!(
			Some(&SkillUsage {
				started_holding: HashSet::from([SlotKey(42)]),
				holding: HashSet::from([SlotKey(42)]),
				refreshed: HashSet::from([SlotKey(42)]),
			}),
			app.world().entity(entity).get::<SkillUsage>(),
		);
		Ok(())
	}

	#[test]
	fn start_holding_when_no_skill_usage_originally_present() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: SkillParamMut| {
				let mut ctx =
					SkillParamMut::get_context_mut(&mut p, SkillControl { entity }).unwrap();
				ctx.holding(SlotKey(42));
			})?;

		assert_eq!(
			Some(&SkillUsage {
				started_holding: HashSet::from([SlotKey(42)]),
				holding: HashSet::from([SlotKey(42)]),
				refreshed: HashSet::from([SlotKey(42)]),
			}),
			app.world().entity(entity).get::<SkillUsage>(),
		);
		Ok(())
	}

	#[test]
	fn start_holding_insert() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(SkillUsage {
				started_holding: HashSet::from([SlotKey(11)]),
				holding: HashSet::from([SlotKey(11)]),
				refreshed: HashSet::from([SlotKey(11)]),
			})
			.id();

		app.world_mut()
			.run_system_once(move |mut p: SkillParamMut| {
				let mut ctx =
					SkillParamMut::get_context_mut(&mut p, SkillControl { entity }).unwrap();
				ctx.holding(SlotKey(42));
			})?;

		assert_eq!(
			Some(&SkillUsage {
				started_holding: HashSet::from([SlotKey(11), SlotKey(42)]),
				holding: HashSet::from([SlotKey(11), SlotKey(42)]),
				refreshed: HashSet::from([SlotKey(11), SlotKey(42)]),
			}),
			app.world().entity(entity).get::<SkillUsage>(),
		);
		Ok(())
	}

	#[test]
	fn keep_holding_just_after_started() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(SkillUsage {
				started_holding: HashSet::from([SlotKey(42)]),
				holding: HashSet::from([SlotKey(42)]),
				refreshed: HashSet::from([SlotKey(42)]),
			})
			.id();

		app.world_mut()
			.run_system_once(move |mut p: SkillParamMut| {
				let mut ctx =
					SkillParamMut::get_context_mut(&mut p, SkillControl { entity }).unwrap();
				ctx.holding(SlotKey(42));
			})?;

		assert_eq!(
			Some(&SkillUsage {
				started_holding: HashSet::from([]),
				holding: HashSet::from([SlotKey(42)]),
				refreshed: HashSet::from([SlotKey(42)]),
			}),
			app.world().entity(entity).get::<SkillUsage>(),
		);
		Ok(())
	}

	#[test]
	fn keep_holding() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(SkillUsage {
				started_holding: HashSet::from([]),
				holding: HashSet::from([SlotKey(42)]),
				refreshed: HashSet::from([SlotKey(42)]),
			})
			.id();

		app.world_mut()
			.run_system_once(move |mut p: SkillParamMut| {
				let mut ctx =
					SkillParamMut::get_context_mut(&mut p, SkillControl { entity }).unwrap();
				ctx.holding(SlotKey(42));
			})?;

		assert_eq!(
			Some(&SkillUsage {
				started_holding: HashSet::from([]),
				holding: HashSet::from([SlotKey(42)]),
				refreshed: HashSet::from([SlotKey(42)]),
			}),
			app.world().entity(entity).get::<SkillUsage>(),
		);
		Ok(())
	}
}
