use crate::{
	components::skill_button::{DropdownTrigger, SkillButton},
	traits::InsertContentOn,
};
use bevy::prelude::*;
use common::traits::{handles_combo_menu::IsCompatible, thread_safe::ThreadSafe};

impl<T> VisualizeInvalidSkill for T {}

pub(crate) trait VisualizeInvalidSkill {
	fn visualize_invalid_skill<TSkill, TCompatibleChecker>(
		mut commands: Commands,
		buttons: Query<(Entity, &Button<TSkill>), Added<Button<TSkill>>>,
		compatible: Res<TCompatibleChecker>,
	) where
		Self: InsertContentOn + Sized,
		TSkill: ThreadSafe,
		TCompatibleChecker: IsCompatible<TSkill> + Resource,
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
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::slot_key::{Side, SlotKey},
		traits::nested_mock::NestedMocks,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Debug, PartialEq)]
	struct _Skill;

	#[derive(Resource, NestedMocks)]
	struct _Compatible {
		mock: Mock_Compatible,
	}

	#[automock]
	impl IsCompatible<_Skill> for _Compatible {
		fn is_compatible(&self, key: &SlotKey, skill: &_Skill) -> bool {
			self.mock.is_compatible(key, skill)
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Visualization;

	impl InsertContentOn for _Visualization {
		fn insert_content_on(entity: &mut EntityCommands) {
			entity.insert(_Visualization);
		}
	}

	fn setup(compatible: _Compatible) -> App {
		let mut app = App::new().single_threaded(Update);
		app.insert_resource(compatible);
		app.add_systems(
			Update,
			_Visualization::visualize_invalid_skill::<_Skill, _Compatible>,
		);

		app
	}

	#[test]
	fn visualize_unusable() {
		let mut app = setup(_Compatible::new().with_mock(|mock| {
			mock.expect_is_compatible()
				.with(eq(SlotKey::BottomHand(Side::Right)), eq(_Skill))
				.return_const(false);
		}));
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

		app.update();

		let skill = app.world().entity(skill);
		assert_eq!(Some(&_Visualization), skill.get::<_Visualization>());
	}

	#[test]
	fn do_not_visualize_usable() {
		let mut app = setup(_Compatible::new().with_mock(|mock| {
			mock.expect_is_compatible()
				.with(eq(SlotKey::BottomHand(Side::Right)), eq(_Skill))
				.return_const(true);
		}));
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

		app.update();

		let skill = app.world().entity(skill);
		assert_eq!(None, skill.get::<_Visualization>());
	}

	#[test]
	fn do_not_visualize_when_not_added() {
		let mut app = setup(_Compatible::new().with_mock(|mock| {
			mock.expect_is_compatible()
				.with(eq(SlotKey::BottomHand(Side::Right)), eq(_Skill))
				.return_const(true);
		}));
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

		app.update();
		app.world_mut().entity_mut(skill).remove::<_Visualization>();
		app.update();

		let skill = app.world().entity(skill);
		assert_eq!(None, skill.get::<_Visualization>())
	}
}
