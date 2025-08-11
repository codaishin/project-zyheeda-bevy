use crate::{
	components::combo_skill_button::{ComboSkillButton, DropdownTrigger},
	traits::InsertContentOn,
};
use bevy::prelude::*;
use common::traits::{handles_combo_menu::GetComboAblePlayerSkills, thread_safe::ThreadSafe};

impl<T> VisualizeInvalidSkill for T {}

pub(crate) trait VisualizeInvalidSkill {
	fn visualize_invalid_skill<TSkill, TCompatibleChecker>(
		mut commands: Commands,
		buttons: Query<(Entity, &Button<TSkill>), Added<Button<TSkill>>>,
		compatible: Res<TCompatibleChecker>,
	) where
		Self: InsertContentOn + Sized,
		TSkill: PartialEq + Clone + ThreadSafe,
		TCompatibleChecker: GetComboAblePlayerSkills<TSkill> + Resource,
	{
		let visualize = Self::insert_content_on;

		for (entity, button) in &buttons {
			let Some(key) = button.key_path.last() else {
				continue;
			};
			let compatible_skills = compatible.get_combo_able_player_skills(key);
			if compatible_skills.contains(&button.skill) {
				continue;
			}
			let Ok(mut entity) = commands.get_entity(entity) else {
				continue;
			};
			visualize(&mut entity);
		}
	}
}

type Button<TSkill> = ComboSkillButton<DropdownTrigger, TSkill>;

#[cfg(test)]
mod tests {
	use super::*;
	use common::tools::action_key::slot::{PlayerSlot, Side};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Debug, PartialEq, Default, Clone)]
	struct _Skill(&'static str);

	#[derive(Resource, NestedMocks)]
	struct _Compatible {
		mock: Mock_Compatible,
	}

	#[automock]
	impl GetComboAblePlayerSkills<_Skill> for _Compatible {
		fn get_combo_able_player_skills(&self, key: &PlayerSlot) -> Vec<_Skill> {
			self.mock.get_combo_able_player_skills(key)
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
			mock.expect_get_combo_able_player_skills()
				.with(eq(PlayerSlot::Lower(Side::Right)))
				.return_const(vec![_Skill("compatible")]);
		}));
		let skill = app
			.world_mut()
			.spawn(ComboSkillButton::<DropdownTrigger, _Skill>::new(
				_Skill("incompatible"),
				vec![
					PlayerSlot::Lower(Side::Left),
					PlayerSlot::Lower(Side::Right),
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
			mock.expect_get_combo_able_player_skills()
				.with(eq(PlayerSlot::Lower(Side::Right)))
				.return_const(vec![_Skill("compatible")]);
		}));
		let skill = app
			.world_mut()
			.spawn(ComboSkillButton::<DropdownTrigger, _Skill>::new(
				_Skill("compatible"),
				vec![
					PlayerSlot::Lower(Side::Left),
					PlayerSlot::Lower(Side::Right),
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
			mock.expect_get_combo_able_player_skills()
				.with(eq(PlayerSlot::Lower(Side::Right)))
				.return_const(vec![_Skill("compatible")]);
		}));
		let skill = app
			.world_mut()
			.spawn(ComboSkillButton::<DropdownTrigger, _Skill>::new(
				_Skill("incompatible"),
				vec![
					PlayerSlot::Lower(Side::Left),
					PlayerSlot::Lower(Side::Right),
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
