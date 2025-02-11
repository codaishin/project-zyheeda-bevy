use crate::{
	components::skill_button::{DropdownTrigger, SkillButton},
	traits::InsertContentOn,
};
use bevy::prelude::*;
use common::traits::{handles_combo_menu::IsCompatible, thread_safe::ThreadSafe};

impl<T> VisualizeInvalidSkill for T {}

pub(crate) trait VisualizeInvalidSkill {
	fn visualize_invalid_skill<TSkill, TCompatibleChecker>(
		In(compatible): In<TCompatibleChecker>,
		mut commands: Commands,
		buttons: Query<(Entity, &Button<TSkill>), Added<Button<TSkill>>>,
	) where
		Self: InsertContentOn + Sized,
		TSkill: ThreadSafe,
		TCompatibleChecker: IsCompatible<TSkill>,
	{
		let visualize = Self::insert_content_on;

		for (entity, button) in &buttons {
			let Some(key) = button.key_path.last() else {
				continue;
			};
			if compatible.is_compatible(key, &button.skill) {
				continue;
			}
			let Some(mut entity) = commands.get_entity(entity) else {
				continue;
			};
			visualize(&mut entity);
		}
	}
}

type Button<TSkill> = SkillButton<DropdownTrigger, TSkill>;

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		simple_init,
		test_tools::utils::SingleThreadedApp,
		tools::slot_key::{Side, SlotKey},
		traits::mock::Mock,
	};
	use mockall::{mock, predicate::eq};

	#[derive(Debug, PartialEq)]
	struct _Skill;

	mock! {
		 _Compatible {}
		impl IsCompatible<_Skill> for _Compatible {
			fn is_compatible(&self, key: &SlotKey, skill: &_Skill) -> bool;
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Visualization;

	impl InsertContentOn for _Visualization {
		fn insert_content_on(entity: &mut EntityCommands) {
			entity.insert(_Visualization);
		}
	}

	simple_init!(Mock_Compatible);

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn visualize_unusable() -> Result<(), RunSystemError> {
		let mut app = setup();
		let compatible = Mock_Compatible::new_mock(|mock| {
			mock.expect_is_compatible()
				.with(eq(SlotKey::BottomHand(Side::Right)), eq(_Skill))
				.return_const(false);
		});
		let skill = app
			.world_mut()
			.spawn(SkillButton::<DropdownTrigger, _Skill>::new(
				_Skill,
				vec![
					SlotKey::BottomHand(Side::Left),
					SlotKey::BottomHand(Side::Right),
				],
			))
			.id();

		app.world_mut().run_system_once_with(
			compatible,
			_Visualization::visualize_invalid_skill::<_Skill, Mock_Compatible>,
		)?;

		let skill = app.world().entity(skill);
		assert_eq!(Some(&_Visualization), skill.get::<_Visualization>());
		Ok(())
	}

	#[test]
	fn do_not_visualize_usable() -> Result<(), RunSystemError> {
		let mut app = setup();
		let compatible = Mock_Compatible::new_mock(|mock| {
			mock.expect_is_compatible()
				.with(eq(SlotKey::BottomHand(Side::Right)), eq(_Skill))
				.return_const(true);
		});
		let skill = app
			.world_mut()
			.spawn(SkillButton::<DropdownTrigger, _Skill>::new(
				_Skill,
				vec![
					SlotKey::BottomHand(Side::Left),
					SlotKey::BottomHand(Side::Right),
				],
			))
			.id();

		app.world_mut().run_system_once_with(
			compatible,
			_Visualization::visualize_invalid_skill::<_Skill, Mock_Compatible>,
		)?;

		let skill = app.world().entity(skill);
		assert_eq!(None, skill.get::<_Visualization>());
		Ok(())
	}

	#[test]
	fn do_not_visualize_when_not_added() {
		let mut app = setup();
		let get_compatible = || {
			Mock_Compatible::new_mock(|mock| {
				mock.expect_is_compatible()
					.with(eq(SlotKey::BottomHand(Side::Right)), eq(_Skill))
					.return_const(true);
			})
		};
		let skill = app
			.world_mut()
			.spawn(SkillButton::<DropdownTrigger, _Skill>::new(
				_Skill,
				vec![
					SlotKey::BottomHand(Side::Left),
					SlotKey::BottomHand(Side::Right),
				],
			))
			.id();
		app.add_systems(
			Update,
			get_compatible.pipe(_Visualization::visualize_invalid_skill::<_Skill, Mock_Compatible>),
		);

		app.update();
		app.world_mut().entity_mut(skill).remove::<_Visualization>();
		app.update();

		let skill = app.world().entity(skill);
		assert_eq!(None, skill.get::<_Visualization>())
	}
}
