use crate::{
	components::combo_skill_button::{ComboSkillButton, DropdownTrigger},
	traits::InsertContentOn,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{EntityContext, TryApplyOn},
		handles_loadout::{AvailableSkills, GetSkillId, ReadAvailableSkills},
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};
use std::fmt::Debug;

impl<TId> ComboSkillButton<DropdownTrigger, TId>
where
	TId: Debug + PartialEq + Clone + ThreadSafe,
{
	pub(crate) fn visualize_invalid<TVisualize, TAgent, TLoadout>(
		mut commands: ZyheedaCommands,
		buttons: Query<(Entity, &Self), Added<Self>>,
		agents: Query<Entity, With<TAgent>>,
		param: StaticSystemParam<TLoadout>,
	) where
		TVisualize: InsertContentOn,
		TAgent: Component,
		TLoadout: for<'c> EntityContext<AvailableSkills, TContext<'c>: ReadAvailableSkills<TId>>,
	{
		for agent in &agents {
			let Some(ctx) = TLoadout::get_entity_context(&param, agent, AvailableSkills) else {
				continue;
			};

			for (entity, button) in &buttons {
				let Some(key) = button.key_path.last() else {
					continue;
				};
				let mut skills = ctx.get_available_skills(*key);

				if skills.any(|skill| skill.get_skill_id() == button.skill.id) {
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
	use crate::components::combo_overview::ComboSkill;
	use common::{
		tools::action_key::slot::{PlayerSlot, SlotKey},
		traits::{
			accessors::get::GetProperty,
			handles_loadout::loadout::{SkillIcon, SkillToken},
			handles_localization::Token,
		},
		zyheeda_commands::ZyheedaEntityCommands,
	};
	use std::{collections::HashMap, sync::LazyLock};
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Agent;

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
	struct _Skill(&'static str);

	impl GetSkillId<&'static str> for _Skill {
		fn get_skill_id(&self) -> &'static str {
			self.0
		}
	}

	const IMAGE: Handle<Image> = Handle::Weak(AssetId::Uuid {
		uuid: AssetId::<Image>::DEFAULT_UUID,
	});

	impl GetProperty<SkillIcon> for _Skill {
		fn get_property(&self) -> &'_ Handle<Image> {
			&IMAGE
		}
	}

	static TOKEN: LazyLock<Token> = LazyLock::new(|| Token::from("my skill token"));

	impl GetProperty<SkillToken> for _Skill {
		fn get_property(&self) -> &Token {
			&TOKEN
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Visualization;

	impl InsertContentOn for _Visualization {
		fn insert_content_on(entity: &mut ZyheedaEntityCommands) {
			entity.try_insert(_Visualization);
		}
	}

	fn combo_skill(id: &'static str) -> ComboSkill<&'static str> {
		ComboSkill {
			id,
			token: Token::from("my combo skill"),
			icon: Handle::default(),
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			ComboSkillButton::<DropdownTrigger, &'static str>::visualize_invalid::<
				_Visualization,
				_Agent,
				Query<Ref<_Slots>>,
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
			.spawn(ComboSkillButton::<DropdownTrigger, &'static str>::new(
				combo_skill("incompatible"),
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
			.spawn(ComboSkillButton::<DropdownTrigger, &'static str>::new(
				combo_skill("compatible"),
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
			.spawn(ComboSkillButton::<DropdownTrigger, &'static str>::new(
				combo_skill("incompatible"),
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
			.spawn(ComboSkillButton::<DropdownTrigger, &'static str>::new(
				combo_skill("incompatible"),
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
