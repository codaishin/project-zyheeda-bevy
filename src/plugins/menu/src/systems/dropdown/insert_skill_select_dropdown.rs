use crate::components::{
	SkillSelectDropdownCommand,
	combo_overview::ComboSkill,
	combo_skill_button::{ComboSkillButton, DropdownItem},
	dropdown::Dropdown,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{DynProperty, GetContext, TryApplyOn},
		handles_loadout::{
			available_skills::{AvailableSkills, ReadAvailableSkills},
			skills::{GetSkillId, SkillIcon, SkillToken},
		},
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::fmt::Debug;

impl<TLayout> SkillSelectDropdownCommand<TLayout> {
	pub(crate) fn insert_dropdown<TAgent, TLoadout, TId>(
		mut commands: ZyheedaCommands,
		dropdown_commands: Query<(Entity, &Self)>,
		agents: Query<Entity, With<TAgent>>,
		param: StaticSystemParam<TLoadout>,
	) where
		TLayout: ThreadSafe + Sized,
		TAgent: Component,
		TId: Debug + PartialEq + Clone + ThreadSafe,
		TLoadout: for<'c> GetContext<AvailableSkills, TContext<'c>: ReadAvailableSkills<TId>>,
	{
		for entity in &agents {
			for (dropdown_entity, command) in &dropdown_commands {
				let Some(key) = command.key_path.last() else {
					continue;
				};
				let Some(ctx) = TLoadout::get_context(&param, AvailableSkills { entity }) else {
					continue;
				};
				let items = ctx
					.get_available_skills(*key)
					.map(|skill| {
						ComboSkillButton::<DropdownItem<TLayout>, TId>::new(
							ComboSkill {
								id: skill.get_skill_id(),
								token: skill.dyn_property::<SkillToken>().clone(),
								icon: skill.dyn_property::<SkillIcon>().clone(),
							},
							command.key_path.clone(),
						)
					})
					.collect::<Vec<_>>();

				commands.try_apply_on(&dropdown_entity, |mut e| {
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
	use common::{
		tools::action_key::slot::{PlayerSlot, Side, SlotKey},
		traits::{accessors::get::GetProperty, handles_localization::Token},
	};
	use std::collections::HashMap;
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Component)]
	struct _Agent;

	#[derive(Debug, PartialEq)]
	struct _Layout;

	#[derive(Component, Clone)]
	struct _Slots(HashMap<SlotKey, Vec<_Skill>>);

	impl ReadAvailableSkills<&'static str> for _Slots {
		type TSkill<'a>
			= _Skill
		where
			Self: 'a;

		fn get_available_skills(&self, key: SlotKey) -> impl Iterator<Item = Self::TSkill<'_>> {
			match self.0.get(&key) {
				Some(skills) => skills.clone().into_iter(),
				None => vec![].into_iter(),
			}
		}
	}

	#[derive(Debug, PartialEq, Default, Clone)]
	struct _Skill {
		id: &'static str,
		token: Token,
		icon: Handle<Image>,
	}

	impl GetSkillId<&'static str> for _Skill {
		fn get_skill_id(&self) -> &'static str {
			self.id
		}
	}

	impl GetProperty<SkillToken> for _Skill {
		fn get_property(&self) -> &Token {
			&self.token
		}
	}

	impl GetProperty<SkillIcon> for _Skill {
		fn get_property(&self) -> &'_ Handle<Image> {
			&self.icon
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
			SkillSelectDropdownCommand::<_Layout>::insert_dropdown::<
				_Agent,
				Query<Ref<_Slots>>,
				&'static str,
			>,
		);

		app
	}

	#[test]
	fn list_compatible_skills() {
		let icon = new_handle();
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
				vec![_Skill {
					id: "my skill id",
					token: Token::from("my skill"),
					icon: icon.clone(),
				}],
			)])),
		));

		app.update();

		assert_eq!(
			Some(&Dropdown {
				items: vec![
					ComboSkillButton::<DropdownItem<_Layout>, &'static str>::new(
						ComboSkill {
							id: "my skill id",
							token: Token::from("my skill"),
							icon
						},
						vec![SlotKey::from(PlayerSlot::Lower(Side::Right))],
					)
				]
			}),
			app.world()
				.entity(dropdown)
				.get::<Dropdown<ComboSkillButton<DropdownItem<_Layout>, &'static str>>>()
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
			vec![_Skill {
				id: "my skill id",
				token: Token::from("my skill"),
				icon: new_handle(),
			}],
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
