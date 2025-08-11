use crate::components::{
	SkillSelectDropdownInsertCommand,
	combo_skill_button::{ComboSkillButton, DropdownItem},
	dropdown::Dropdown,
};
use bevy::prelude::*;
use common::{
	tools::action_key::slot::PlayerSlot,
	traits::{
		accessors::get::TryApplyOn,
		handles_combo_menu::GetComboAblePlayerSkills,
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl<T> DropdownSkillSelectInsert for T {}

pub(crate) trait DropdownSkillSelectInsert {
	fn dropdown_skill_select_insert<TSkill, TIsCompatible>(
		mut commands: ZyheedaCommands,
		dropdown_commands: Query<(Entity, &SkillSelectDropdownInsertCommand<PlayerSlot, Self>)>,
		compatible: Res<TIsCompatible>,
	) where
		Self: ThreadSafe + Sized,
		TSkill: Clone + ThreadSafe,
		TIsCompatible: GetComboAblePlayerSkills<TSkill> + Resource,
	{
		for (entity, command) in &dropdown_commands {
			commands.try_apply_on(&entity, |mut e| {
				if let Some(items) = compatible_skills(command, compatible.as_ref()) {
					e.try_insert(Dropdown { items });
				}
				e.try_remove::<SkillSelectDropdownInsertCommand<PlayerSlot, Self>>();
			});
		}
	}
}

fn compatible_skills<TLayout, TSkill, TIsCompatible>(
	command: &SkillSelectDropdownInsertCommand<PlayerSlot, TLayout>,
	compatible: &TIsCompatible,
) -> Option<Vec<ComboSkillButton<DropdownItem<TLayout>, TSkill>>>
where
	TLayout: ThreadSafe,
	TSkill: Clone,
	TIsCompatible: GetComboAblePlayerSkills<TSkill>,
{
	let key = command.key_path.last()?;
	let skills = compatible
		.get_combo_able_player_skills(key)
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
	impl GetComboAblePlayerSkills<_Skill> for _ComboAbleSkills {
		fn get_combo_able_player_skills(&self, key: &PlayerSlot) -> Vec<_Skill> {
			self.mock.get_combo_able_player_skills(key)
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
			mock.expect_get_combo_able_player_skills()
				.with(eq(PlayerSlot::Lower(Side::Right)))
				.return_const(vec![_Skill("my skill")]);
		}));
		let dropdown = app
			.world_mut()
			.spawn(
				SkillSelectDropdownInsertCommand::<PlayerSlot, _Layout>::new(vec![
					PlayerSlot::Lower(Side::Right),
				]),
			)
			.id();

		app.update();

		assert_eq!(
			Some(&Dropdown {
				items: vec![ComboSkillButton::<DropdownItem<_Layout>, _Skill>::new(
					_Skill("my skill"),
					vec![PlayerSlot::Lower(Side::Right)],
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
			mock.expect_get_combo_able_player_skills()
				.with(eq(PlayerSlot::Lower(Side::Right)))
				.return_const(vec![]);
		}));
		let dropdown = app
			.world_mut()
			.spawn(
				SkillSelectDropdownInsertCommand::<PlayerSlot, _Layout>::new(vec![
					PlayerSlot::Lower(Side::Right),
				]),
			)
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
