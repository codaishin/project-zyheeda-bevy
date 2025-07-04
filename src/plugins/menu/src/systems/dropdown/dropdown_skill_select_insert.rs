use crate::components::{
	SkillSelectDropdownInsertCommand,
	combo_skill_button::{ComboSkillButton, DropdownItem},
	dropdown::Dropdown,
};
use bevy::prelude::*;
use common::{
	tools::action_key::slot::SlotKey,
	traits::{
		handles_combo_menu::GetComboAbleSkills,
		thread_safe::ThreadSafe,
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
	},
};

impl<T> DropdownSkillSelectInsert for T {}

pub(crate) trait DropdownSkillSelectInsert {
	fn dropdown_skill_select_insert<TSkill, TIsCompatible>(
		mut commands: Commands,
		dropdown_commands: Query<(Entity, &SkillSelectDropdownInsertCommand<SlotKey, Self>)>,
		compatible: Res<TIsCompatible>,
	) where
		Self: ThreadSafe + Sized,
		TSkill: Clone + ThreadSafe,
		TIsCompatible: GetComboAbleSkills<TSkill> + Resource,
	{
		for (entity, command) in &dropdown_commands {
			if let Some(items) = compatible_skills(command, compatible.as_ref()) {
				commands.try_insert_on(entity, Dropdown { items });
			}
			commands.try_remove_from::<SkillSelectDropdownInsertCommand<SlotKey, Self>>(entity);
		}
	}
}

fn compatible_skills<TLayout, TSkill, TIsCompatible>(
	command: &SkillSelectDropdownInsertCommand<SlotKey, TLayout>,
	compatible: &TIsCompatible,
) -> Option<Vec<ComboSkillButton<DropdownItem<TLayout>, TSkill>>>
where
	TLayout: ThreadSafe,
	TSkill: Clone,
	TIsCompatible: GetComboAbleSkills<TSkill>,
{
	let key = command.key_path.last()?;
	let skills = compatible
		.get_combo_able_skills(key)
		.iter()
		.map(|skill| {
			ComboSkillButton::<DropdownItem<TLayout>, TSkill>::new(
				skill.clone(),
				command.key_path.clone(),
			)
		})
		.collect::<Vec<_>>();

	Some(skills)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::dropdown::Dropdown;
	use common::tools::action_key::slot::Side;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Debug, PartialEq, Default, Clone)]
	struct _Skill(&'static str);

	#[derive(Debug, PartialEq)]
	struct _Layout;

	#[derive(Resource, NestedMocks)]
	struct _ComboAbleSkills {
		mock: Mock_ComboAbleSkills,
	}

	#[automock]
	impl GetComboAbleSkills<_Skill> for _ComboAbleSkills {
		fn get_combo_able_skills(&self, key: &SlotKey) -> Vec<_Skill> {
			self.mock.get_combo_able_skills(key)
		}
	}

	#[derive(Debug, PartialEq, Clone)]
	enum _DropdownKey {
		None,
		Ok,
	}

	fn setup(skills: _ComboAbleSkills) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			_Layout::dropdown_skill_select_insert::<_Skill, _ComboAbleSkills>,
		);
		app.insert_resource(skills);

		app
	}

	#[test]
	fn list_compatible_skills() {
		let mut app = setup(_ComboAbleSkills::new().with_mock(|mock| {
			mock.expect_get_combo_able_skills()
				.with(eq(SlotKey::BottomHand(Side::Right)))
				.return_const(vec![_Skill("my skill")]);
		}));
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownInsertCommand::<SlotKey, _Layout>::new(
				vec![SlotKey::BottomHand(Side::Right)],
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Dropdown {
				items: vec![ComboSkillButton::<DropdownItem<_Layout>, _Skill>::new(
					_Skill("my skill"),
					vec![SlotKey::BottomHand(Side::Right)],
				)]
			}),
			app.world()
				.entity(dropdown)
				.get::<Dropdown<ComboSkillButton<DropdownItem<_Layout>, _Skill>>>()
		);
	}

	#[test]
	fn remove_command() {
		let mut app = setup(_ComboAbleSkills::new().with_mock(|mock| {
			mock.expect_get_combo_able_skills()
				.with(eq(SlotKey::BottomHand(Side::Right)))
				.return_const(vec![]);
		}));
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownInsertCommand::<SlotKey, _Layout>::new(
				vec![SlotKey::BottomHand(Side::Right)],
			))
			.id();

		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(dropdown)
				.get::<SkillSelectDropdownInsertCommand<_Skill, _Layout>>()
		);
	}
}
