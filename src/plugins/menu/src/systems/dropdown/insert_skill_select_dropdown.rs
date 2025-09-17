use crate::components::{
	SkillSelectDropdownCommand,
	combo_skill_button::{ComboSkillButton, DropdownItem},
	dropdown::Dropdown,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::{AssociatedParam, GetParamEntry, TryApplyOn},
		handles_loadout::slot_component::AvailableSkills,
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl<TLayout> SkillSelectDropdownCommand<TLayout> {
	pub(crate) fn insert_dropdown<TAgent, TSlots, TSkills>(
		mut commands: ZyheedaCommands,
		dropdown_commands: Query<(Entity, &Self)>,
		slots: Query<&TSlots, With<TAgent>>,
		param: StaticSystemParam<AssociatedParam<TSlots, AvailableSkills<SlotKey>>>,
	) where
		TLayout: ThreadSafe + Sized,
		TAgent: Component,
		TSkills: IntoIterator,
		TSkills::Item: Clone + ThreadSafe,
		for<'w, 's> TSlots:
			Component + GetParamEntry<'w, 's, AvailableSkills<SlotKey>, TItem = TSkills>,
	{
		for slots in &slots {
			for (entity, command) in &dropdown_commands {
				let Some(key) = command.key_path.last() else {
					continue;
				};
				let Some(items) = slots.get_param_entry(&AvailableSkills(*key), &param) else {
					continue;
				};
				let items = items
					.into_iter()
					.map(|skill| {
						ComboSkillButton::<DropdownItem<TLayout>, TSkills::Item>::new(
							skill.clone(),
							command.key_path.clone(),
						)
					})
					.collect::<Vec<_>>();

				commands.try_apply_on(&entity, |mut e| {
					e.try_insert(Dropdown { items });
					e.try_remove::<Self>();
				});
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::dropdown::Dropdown;
	use common::tools::action_key::slot::{PlayerSlot, Side};
	use std::collections::HashMap;
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Agent;

	#[derive(Debug, PartialEq, Default, Clone)]
	struct _Skill(&'static str);

	#[derive(Debug, PartialEq)]
	struct _Layout;

	#[derive(Component)]
	struct _Slots(HashMap<SlotKey, Vec<_Skill>>);

	impl<'w, 's> GetParamEntry<'w, 's, AvailableSkills<SlotKey>> for _Slots {
		type TParam = ();
		type TItem = Vec<_Skill>;

		fn get_param_entry(
			&self,
			AvailableSkills(key): &AvailableSkills<SlotKey>,
			_: &(),
		) -> Option<Vec<_Skill>> {
			self.0.get(key).cloned()
		}
	}

	#[derive(Debug, PartialEq, Clone)]
	enum _DropdownKey {
		None,
		Ok,
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			SkillSelectDropdownCommand::<_Layout>::insert_dropdown::<_Agent, _Slots, Vec<_Skill>>,
		);

		app
	}

	#[test]
	fn list_compatible_skills() {
		let mut app = setup();
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownCommand::<_Layout>::new(vec![
				SlotKey::from(PlayerSlot::LOWER_R),
			]))
			.id();
		app.world_mut().spawn((
			_Agent,
			_Slots(HashMap::from([(
				SlotKey::from(PlayerSlot::LOWER_R),
				vec![_Skill("my skill")],
			)])),
		));

		app.update();

		assert_eq!(
			Some(&Dropdown {
				items: vec![ComboSkillButton::<DropdownItem<_Layout>, _Skill>::new(
					_Skill("my skill"),
					vec![SlotKey::from(PlayerSlot::Lower(Side::Right))],
				)]
			}),
			app.world()
				.entity(dropdown)
				.get::<Dropdown<ComboSkillButton<DropdownItem<_Layout>, _Skill>>>()
		);
	}

	#[test]
	fn list_no_compatible_skills_when_no_agent() {
		let mut app = setup();
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownCommand::<_Layout>::new(vec![
				SlotKey::from(PlayerSlot::LOWER_R),
			]))
			.id();
		app.world_mut().spawn(_Slots(HashMap::from([(
			SlotKey::from(PlayerSlot::LOWER_R),
			vec![_Skill("my skill")],
		)])));

		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(dropdown)
				.get::<Dropdown<ComboSkillButton<DropdownItem<_Layout>, _Skill>>>()
		);
	}

	#[test]
	fn remove_command() {
		let mut app = setup();
		let dropdown = app
			.world_mut()
			.spawn(SkillSelectDropdownCommand::<_Layout>::new(vec![
				SlotKey::from(PlayerSlot::LOWER_R),
			]))
			.id();
		app.world_mut().spawn((
			_Agent,
			_Slots(HashMap::from([(
				SlotKey::from(PlayerSlot::LOWER_R),
				vec![],
			)])),
		));

		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(dropdown)
				.get::<SkillSelectDropdownCommand<_Layout>>()
		);
	}
}
