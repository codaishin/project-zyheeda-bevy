use crate::{
	components::combo_skill_button::{ComboSkillButton, DropdownTrigger},
	traits::InsertContentOn,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::{GetParamEntry, Param, ParamEntry, TryApplyOn},
		handles_loadout::AvailableSkills,
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl<TSkill> ComboSkillButton<DropdownTrigger, TSkill>
where
	TSkill: ThreadSafe,
{
	pub(crate) fn visualize_invalid<TVisualize, TAgent, TSlots>(
		mut commands: ZyheedaCommands,
		buttons: Query<(Entity, &Self), Added<Self>>,
		slots: Query<&TSlots, With<TAgent>>,
		param: StaticSystemParam<Param<TSlots, AvailableSkills<SlotKey>>>,
	) where
		TVisualize: InsertContentOn,
		TAgent: Component,
		for<'w, 's> TSlots: Component + GetParamEntry<'w, 's, AvailableSkills<SlotKey>>,
		for<'w, 's> ParamEntry<'w, 's, TSlots, AvailableSkills<SlotKey>>:
			IntoIterator<Item = TSkill>,
		TSkill: PartialEq,
	{
		for slots in &slots {
			for (entity, button) in &buttons {
				let Some(key) = button.key_path.last() else {
					continue;
				};
				let is_compatible = slots
					.get_param_entry(&AvailableSkills(*key), &param)
					.into_iter()
					.any(|skill| skill == button.skill);
				if is_compatible {
					continue;
				}

				commands.try_apply_on(&entity, |mut entity| {
					TVisualize::insert_content_on(&mut entity);
				});
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::action_key::slot::{PlayerSlot, SlotKey},
		zyheeda_commands::ZyheedaEntityCommands,
	};
	use std::collections::HashMap;
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Agent;

	#[derive(Debug, PartialEq, Default, Clone)]
	struct _Skill(&'static str);

	#[derive(Component)]
	struct _Slots(HashMap<SlotKey, Vec<_Skill>>);

	impl<'w, 's> GetParamEntry<'w, 's, AvailableSkills<SlotKey>> for _Slots {
		type TParam = ();
		type TEntry = Vec<_Skill>;

		fn get_param_entry(
			&self,
			AvailableSkills(key): &AvailableSkills<SlotKey>,
			_: &(),
		) -> Self::TEntry {
			self.0.get(key).cloned().unwrap_or_default()
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Visualization;

	impl InsertContentOn for _Visualization {
		fn insert_content_on(entity: &mut ZyheedaEntityCommands) {
			entity.try_insert(_Visualization);
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			ComboSkillButton::<DropdownTrigger, _Skill>::visualize_invalid::<
				_Visualization,
				_Agent,
				_Slots,
			>,
		);

		app
	}

	#[test]
	fn visualize_unusable() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Slots(HashMap::from([(
				SlotKey::from(PlayerSlot::LOWER_R),
				vec![_Skill("compatible")],
			)])),
		));
		let skill = app
			.world_mut()
			.spawn(ComboSkillButton::<DropdownTrigger, _Skill>::new(
				_Skill("incompatible"),
				vec![
					SlotKey::from(PlayerSlot::LOWER_L),
					SlotKey::from(PlayerSlot::LOWER_R),
				],
			))
			.id();

		app.update();

		assert_eq!(
			Some(&_Visualization),
			app.world().entity(skill).get::<_Visualization>()
		);
	}

	#[test]
	fn do_not_visualize_usable() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Slots(HashMap::from([(
				SlotKey::from(PlayerSlot::LOWER_R),
				vec![_Skill("compatible")],
			)])),
		));
		let skill = app
			.world_mut()
			.spawn(ComboSkillButton::<DropdownTrigger, _Skill>::new(
				_Skill("compatible"),
				vec![
					SlotKey::from(PlayerSlot::LOWER_L),
					SlotKey::from(PlayerSlot::LOWER_R),
				],
			))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(skill).get::<_Visualization>());
	}

	#[test]
	fn do_not_visualize_when_not_added() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Slots(HashMap::from([(
				SlotKey::from(PlayerSlot::LOWER_R),
				vec![_Skill("compatible")],
			)])),
		));
		let skill = app
			.world_mut()
			.spawn(ComboSkillButton::<DropdownTrigger, _Skill>::new(
				_Skill("incompatible"),
				vec![
					SlotKey::from(PlayerSlot::LOWER_L),
					SlotKey::from(PlayerSlot::LOWER_R),
				],
			))
			.id();

		app.update();
		app.world_mut().entity_mut(skill).remove::<_Visualization>();
		app.update();

		assert_eq!(None, app.world().entity(skill).get::<_Visualization>())
	}

	#[test]
	fn do_nothing_if_agent_missing() {
		let mut app = setup();
		app.world_mut().spawn(_Slots(HashMap::from([(
			SlotKey::from(PlayerSlot::LOWER_R),
			vec![_Skill("compatible")],
		)])));
		let skill = app
			.world_mut()
			.spawn(ComboSkillButton::<DropdownTrigger, _Skill>::new(
				_Skill("incompatible"),
				vec![
					SlotKey::from(PlayerSlot::LOWER_L),
					SlotKey::from(PlayerSlot::LOWER_R),
				],
			))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(skill).get::<_Visualization>());
	}
}
