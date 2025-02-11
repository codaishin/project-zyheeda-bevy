use crate::components::{
	dropdown::Dropdown,
	skill_button::{DropdownItem, SkillButton},
	SkillSelectDropdownInsertCommand,
};
use bevy::prelude::*;
use common::{
	tools::slot_key::SlotKey,
	traits::{
		handles_combo_menu::IsCompatible,
		thread_safe::ThreadSafe,
		try_insert_on::TryInsertOn,
		try_remove_from::TryRemoveFrom,
	},
};

impl<T> DropdownSkillSelectInsert for T {}

pub(crate) trait DropdownSkillSelectInsert {
	fn dropdown_skill_select_insert<TSkill, TIsCompatible>(
		In(compatible): In<TIsCompatible>,
		mut commands: Commands,
		dropdown_commands: Query<(Entity, &SkillSelectDropdownInsertCommand<SlotKey, Self>)>,
		skills: Res<Assets<TSkill>>,
	) where
		Self: ThreadSafe + Sized,
		TSkill: Asset + PartialEq + Clone,
		TIsCompatible: IsCompatible<TSkill>,
	{
		for (entity, command) in &dropdown_commands {
			if let Some(items) = compatible_skills(command, &compatible, &skills) {
				commands.try_insert_on(entity, Dropdown { items });
			}
			commands.try_remove_from::<SkillSelectDropdownInsertCommand<SlotKey, Self>>(entity);
		}
	}
}

fn compatible_skills<TLayout, TSkill, TIsCompatible>(
	command: &SkillSelectDropdownInsertCommand<SlotKey, TLayout>,
	compatible: &TIsCompatible,
	skills: &Assets<TSkill>,
) -> Option<Vec<SkillButton<DropdownItem<TLayout>, TSkill>>>
where
	TLayout: ThreadSafe,
	TSkill: Asset + PartialEq + Clone,
	TIsCompatible: IsCompatible<TSkill>,
{
	let key = command.key_path.last()?;
	let mut buffer = Vec::new();
	let skills = skills
		.iter()
		.filter(|(_, skill)| compatible.is_compatible(key, skill) && !seen(&mut buffer, skill))
		.map(|(_, skill)| {
			SkillButton::<DropdownItem<TLayout>, TSkill>::new(
				skill.clone(),
				command.key_path.clone(),
			)
		})
		.collect::<Vec<_>>();

	Some(skills)
}

fn seen<TSkill>(buffer: &mut Vec<TSkill>, skill: &TSkill) -> bool
where
	TSkill: PartialEq + Clone,
{
	if buffer.contains(skill) {
		return true;
	}

	buffer.push(skill.clone());
	false
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::dropdown::Dropdown;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::{
		simple_init,
		test_tools::utils::SingleThreadedApp,
		tools::slot_key::Side,
		traits::mock::Mock,
	};
	use mockall::{mock, predicate::eq};

	#[derive(Asset, TypePath, Debug, PartialEq, Clone)]
	struct _Skill(&'static str);

	#[derive(Debug, PartialEq)]
	struct _Layout;

	mock! {
		_Compatible {}
		impl IsCompatible<_Skill> for _Compatible {
			fn is_compatible(&self, key: &SlotKey, skill: &_Skill) -> bool;
		}
	}

	simple_init!(Mock_Compatible);

	#[derive(Debug, PartialEq, Clone)]
	enum _DropdownKey {
		None,
		Ok,
	}

	fn setup<const N: usize>(skills: [_Skill; N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut skill_assets = Assets::<_Skill>::default();

		for skill in skills {
			skill_assets.add(skill);
		}

		app.insert_resource(skill_assets);

		app
	}

	#[test]
	fn list_compatible_skills() -> Result<(), RunSystemError> {
		let mut app = setup([_Skill("my skill")]);
		let compatible = Mock_Compatible::new_mock(|mock| {
			mock.expect_is_compatible()
				.with(eq(SlotKey::BottomHand(Side::Right)), eq(_Skill("my skill")))
				.return_const(true);
		});
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownInsertCommand::<SlotKey, _Layout>::new(
				vec![SlotKey::BottomHand(Side::Right)],
			))
			.id();

		app.world_mut().run_system_once_with(
			compatible,
			_Layout::dropdown_skill_select_insert::<_Skill, Mock_Compatible>,
		)?;

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			Some(&Dropdown {
				items: vec![SkillButton::<DropdownItem<_Layout>, _Skill>::new(
					_Skill("my skill"),
					vec![SlotKey::BottomHand(Side::Right)],
				)]
			}),
			dropdown.get::<Dropdown<SkillButton<DropdownItem<_Layout>, _Skill>>>()
		);
		Ok(())
	}

	#[test]
	fn do_not_list_incompatible_skills() -> Result<(), RunSystemError> {
		let mut app = setup([_Skill("my skill")]);
		let compatible = Mock_Compatible::new_mock(|mock| {
			mock.expect_is_compatible()
				.with(eq(SlotKey::BottomHand(Side::Right)), eq(_Skill("my skill")))
				.return_const(false);
		});
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownInsertCommand::<SlotKey, _Layout>::new(
				vec![SlotKey::BottomHand(Side::Right)],
			))
			.id();

		app.world_mut().run_system_once_with(
			compatible,
			_Layout::dropdown_skill_select_insert::<_Skill, Mock_Compatible>,
		)?;

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			Some(&Dropdown { items: vec![] }),
			dropdown.get::<Dropdown<SkillButton<DropdownItem<_Layout>, _Skill>>>()
		);
		Ok(())
	}

	#[test]
	fn list_unique_skills() -> Result<(), RunSystemError> {
		let mut app = setup([_Skill("my skill"), _Skill("my skill")]);
		let compatible = Mock_Compatible::new_mock(|mock| {
			mock.expect_is_compatible()
				.with(eq(SlotKey::BottomHand(Side::Right)), eq(_Skill("my skill")))
				.return_const(true);
		});
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownInsertCommand::<SlotKey, _Layout>::new(
				vec![SlotKey::BottomHand(Side::Right)],
			))
			.id();

		app.world_mut().run_system_once_with(
			compatible,
			_Layout::dropdown_skill_select_insert::<_Skill, Mock_Compatible>,
		)?;

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			Some(&Dropdown {
				items: vec![SkillButton::<DropdownItem<_Layout>, _Skill>::new(
					_Skill("my skill"),
					vec![SlotKey::BottomHand(Side::Right)],
				)]
			}),
			dropdown.get::<Dropdown<SkillButton<DropdownItem<_Layout>, _Skill>>>()
		);
		Ok(())
	}

	#[test]
	fn remove_command() -> Result<(), RunSystemError> {
		let mut app = setup([_Skill("my skill"), _Skill("my skill")]);
		let compatible = Mock_Compatible::new_mock(|mock| {
			mock.expect_is_compatible()
				.with(eq(SlotKey::BottomHand(Side::Right)), eq(_Skill("my skill")))
				.return_const(true);
		});
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownInsertCommand::<SlotKey, _Layout>::new(
				vec![SlotKey::BottomHand(Side::Right)],
			))
			.id();

		app.world_mut().run_system_once_with(
			compatible,
			_Layout::dropdown_skill_select_insert::<_Skill, Mock_Compatible>,
		)?;

		let dropdown = app.world().entity(dropdown);

		assert_eq!(
			None,
			dropdown.get::<SkillSelectDropdownInsertCommand<_Skill, _Layout>>()
		);
		Ok(())
	}
}
