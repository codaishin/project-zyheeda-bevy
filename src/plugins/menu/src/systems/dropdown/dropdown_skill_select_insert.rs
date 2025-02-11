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
		mut commands: Commands,
		dropdown_commands: Query<(Entity, &SkillSelectDropdownInsertCommand<SlotKey, Self>)>,
		skills: Res<Assets<TSkill>>,
		compatible: Res<TIsCompatible>,
	) where
		Self: ThreadSafe + Sized,
		TSkill: Asset + PartialEq + Clone,
		TIsCompatible: IsCompatible<TSkill> + Resource,
	{
		for (entity, command) in &dropdown_commands {
			if let Some(items) = compatible_skills(command, compatible.as_ref(), &skills) {
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
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::slot_key::Side,
		traits::nested_mock::NestedMocks,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Asset, TypePath, Debug, PartialEq, Clone)]
	struct _Skill(&'static str);

	#[derive(Debug, PartialEq)]
	struct _Layout;

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

	#[derive(Debug, PartialEq, Clone)]
	enum _DropdownKey {
		None,
		Ok,
	}

	fn setup<const N: usize>(skills: [_Skill; N], compatible: _Compatible) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut skill_assets = Assets::<_Skill>::default();

		for skill in skills {
			skill_assets.add(skill);
		}

		app.add_systems(
			Update,
			_Layout::dropdown_skill_select_insert::<_Skill, _Compatible>,
		);
		app.insert_resource(compatible);
		app.insert_resource(skill_assets);

		app
	}

	#[test]
	fn list_compatible_skills() {
		let mut app = setup(
			[_Skill("my skill")],
			_Compatible::new().with_mock(|mock| {
				mock.expect_is_compatible()
					.with(eq(SlotKey::BottomHand(Side::Right)), eq(_Skill("my skill")))
					.return_const(true);
			}),
		);
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownInsertCommand::<SlotKey, _Layout>::new(
				vec![SlotKey::BottomHand(Side::Right)],
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Dropdown {
				items: vec![SkillButton::<DropdownItem<_Layout>, _Skill>::new(
					_Skill("my skill"),
					vec![SlotKey::BottomHand(Side::Right)],
				)]
			}),
			app.world()
				.entity(dropdown)
				.get::<Dropdown<SkillButton<DropdownItem<_Layout>, _Skill>>>()
		);
	}

	#[test]
	fn do_not_list_incompatible_skills() {
		let mut app = setup(
			[_Skill("my skill")],
			_Compatible::new().with_mock(|mock| {
				mock.expect_is_compatible()
					.with(eq(SlotKey::BottomHand(Side::Right)), eq(_Skill("my skill")))
					.return_const(false);
			}),
		);
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownInsertCommand::<SlotKey, _Layout>::new(
				vec![SlotKey::BottomHand(Side::Right)],
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Dropdown { items: vec![] }),
			app.world()
				.entity(dropdown)
				.get::<Dropdown<SkillButton<DropdownItem<_Layout>, _Skill>>>()
		);
	}

	#[test]
	fn list_unique_skills() {
		let mut app = setup(
			[_Skill("my skill"), _Skill("my skill")],
			_Compatible::new().with_mock(|mock| {
				mock.expect_is_compatible()
					.with(eq(SlotKey::BottomHand(Side::Right)), eq(_Skill("my skill")))
					.return_const(true);
			}),
		);
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownInsertCommand::<SlotKey, _Layout>::new(
				vec![SlotKey::BottomHand(Side::Right)],
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Dropdown {
				items: vec![SkillButton::<DropdownItem<_Layout>, _Skill>::new(
					_Skill("my skill"),
					vec![SlotKey::BottomHand(Side::Right)],
				)]
			}),
			app.world()
				.entity(dropdown)
				.get::<Dropdown<SkillButton<DropdownItem<_Layout>, _Skill>>>()
		);
	}

	#[test]
	fn remove_command() {
		let mut app = setup(
			[_Skill("my skill"), _Skill("my skill")],
			_Compatible::new().with_mock(|mock| {
				mock.expect_is_compatible()
					.with(eq(SlotKey::BottomHand(Side::Right)), eq(_Skill("my skill")))
					.return_const(true);
			}),
		);
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
