use crate::components::{icon::Icon, label::UILabel, quickbar_panel::QuickbarPanel};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{DynProperty, EntityContext, TryApplyOn},
		handles_loadout::skills::{ReadSkills, SkillIcon, SkillToken, Skills},
		handles_localization::Token,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl QuickbarPanel {
	pub(crate) fn set_icon<TAgent, TLoadout>(
		mut commands: ZyheedaCommands,
		param: StaticSystemParam<TLoadout>,
		panels: Query<PanelComponents>,
		agents: Query<Entity, With<TAgent>>,
	) where
		TAgent: Component,
		TLoadout: for<'c> EntityContext<Skills, TContext<'c>: ReadSkills>,
	{
		for agent in &agents {
			let Some(ctx) = TLoadout::get_entity_context(&param, agent, Skills) else {
				continue;
			};

			for (entity, Self { key, .. }, current_icon, current_label) in &panels {
				let Some(skill) = ctx.get_skill(*key) else {
					continue;
				};
				let token = skill.dyn_property::<SkillToken>();
				let image = skill.dyn_property::<SkillIcon>();

				commands.try_apply_on(&entity, |mut e| {
					if !loaded(current_icon, image) {
						e.try_insert(Icon::Load(image.clone()));
					}

					if !labeled(current_label, token) {
						e.try_insert(UILabel(token.clone()));
					}
				});
			}
		}
	}
}

type PanelComponents<'a> = (
	Entity,
	&'a QuickbarPanel,
	Option<&'a Icon>,
	Option<&'a UILabel<Token>>,
);

fn loaded(icon: Option<&Icon>, image: &Handle<Image>) -> bool {
	matches!(icon, Some(Icon::Loaded(icon_image)) if icon_image == image)
}

fn labeled(label: Option<&UILabel<Token>>, token: &Token) -> bool {
	matches!(label, Some(UILabel(label_token)) if label_token == token)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{icon::Icon, label::UILabel},
		tools::PanelState,
	};
	use common::{
		tools::{action_key::slot::PlayerSlot, skill_execution::SkillExecution},
		traits::{
			accessors::get::GetProperty,
			handles_loadout::LoadoutKey,
			handles_localization::Token,
		},
	};
	use std::collections::HashMap;
	use testing::{IsChanged, SingleThreadedApp, new_handle};

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, Clone)]
	struct _Skills(HashMap<LoadoutKey, _Skill>);

	impl ReadSkills for _Skills {
		type TSkill<'a>
			= _Skill
		where
			Self: 'a;

		fn get_skill<TKey>(&self, key: TKey) -> Option<Self::TSkill<'_>>
		where
			TKey: Into<LoadoutKey>,
		{
			self.0.get(&key.into()).cloned()
		}
	}

	#[derive(Debug, PartialEq, Clone)]
	struct _Skill {
		icon: Handle<Image>,
		token: Token,
	}

	impl GetProperty<SkillIcon> for _Skill {
		fn get_property(&self) -> &'_ Handle<Image> {
			&self.icon
		}
	}

	impl GetProperty<SkillToken> for _Skill {
		fn get_property(&self) -> &'_ Token {
			&self.token
		}
	}

	impl GetProperty<SkillExecution> for _Skill {
		fn get_property(&self) -> SkillExecution {
			SkillExecution::None
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(
				QuickbarPanel::set_icon::<_Agent, Query<Ref<_Skills>>>,
				IsChanged::<UILabel<Token>>::detect,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn insert_icon() {
		let image = new_handle();
		let item = _Skill {
			token: Token::from("my item"),
			icon: image.clone(),
		};
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Skills(HashMap::from([(
				LoadoutKey::from(PlayerSlot::LOWER_R),
				item.clone(),
			)])),
		));
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				key: PlayerSlot::LOWER_R,
				state: PanelState::Empty,
			})
			.id();

		app.update();

		assert_eq!(
			Some(&Icon::Load(image)),
			app.world().entity(panel).get::<Icon>(),
		);
	}

	#[test]
	fn do_not_insert_icon_when_already_loaded() {
		let image = new_handle();
		let item = _Skill {
			token: Token::from("my item"),
			icon: image.clone(),
		};
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Skills(HashMap::from([(
				LoadoutKey::from(PlayerSlot::LOWER_R),
				item.clone(),
			)])),
		));
		let panel = app
			.world_mut()
			.spawn((
				QuickbarPanel {
					key: PlayerSlot::LOWER_R,
					state: PanelState::Empty,
				},
				Icon::Loaded(image.clone()),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&Icon::Loaded(image)),
			app.world().entity(panel).get::<Icon>(),
		);
	}

	#[test]
	fn insert_label() {
		let image = new_handle();
		let item = _Skill {
			token: Token::from("my item"),
			icon: image.clone(),
		};
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Skills(HashMap::from([(
				LoadoutKey::from(PlayerSlot::LOWER_R),
				item.clone(),
			)])),
		));
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				key: PlayerSlot::LOWER_R,
				state: PanelState::Empty,
			})
			.id();

		app.update();

		assert_eq!(
			Some(&UILabel(Token::from("my item"))),
			app.world().entity(panel).get::<UILabel<Token>>(),
		);
	}

	#[test]
	fn do_not_insert_label_when_already_present() {
		let image = new_handle();
		let item = _Skill {
			token: Token::from("my item"),
			icon: image.clone(),
		};
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Skills(HashMap::from([(
				LoadoutKey::from(PlayerSlot::LOWER_R),
				item.clone(),
			)])),
		));
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				key: PlayerSlot::LOWER_R,
				state: PanelState::Empty,
			})
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(panel).get::<IsChanged<UILabel<Token>>>(),
		);
	}

	#[test]
	fn insert_icon_when_not_already_loaded_but_label_is_already_present() {
		let image = new_handle();
		let item = _Skill {
			token: Token::from("my item"),
			icon: image.clone(),
		};
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Skills(HashMap::from([(
				LoadoutKey::from(PlayerSlot::LOWER_R),
				item.clone(),
			)])),
		));
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				key: PlayerSlot::LOWER_R,
				state: PanelState::Empty,
			})
			.id();

		app.update();
		app.world_mut().entity_mut(panel).remove::<Icon>();
		app.update();

		assert_eq!(
			Some(&Icon::Load(image)),
			app.world().entity(panel).get::<Icon>(),
		);
	}

	#[test]
	fn do_nothing_when_slots_lack_agent() {
		let image = new_handle();
		let item = _Skill {
			token: Token::from("my item"),
			icon: image.clone(),
		};
		let mut app = setup();
		app.world_mut().spawn(_Skills(HashMap::from([(
			LoadoutKey::from(PlayerSlot::LOWER_R),
			item.clone(),
		)])));
		let panel = app
			.world_mut()
			.spawn(QuickbarPanel {
				key: PlayerSlot::LOWER_R,
				state: PanelState::Empty,
			})
			.id();

		app.update();

		let panel = app.world().entity(panel);
		assert_eq!(
			(None, None),
			(panel.get::<Icon>(), panel.get::<UILabel::<Token>>()),
		);
	}
}
