use crate::{
	components::{
		ColorOverride,
		dispatch_text_color::DispatchTextColor,
		quickbar_panel::QuickbarPanel,
	},
	traits::colors::ColorConfig,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	tools::skill_execution::SkillExecution,
	traits::{
		accessors::get::{DynProperty, EntityContext, GetProperty},
		handles_input::MouseOverrideActive,
		handles_loadout::{ReadSkills, Skills},
	},
};

impl QuickbarPanel {
	pub(crate) fn set_color<TAgent, TActionKeyButton, TLoadout>(
		commands: Commands,
		buttons: Query<(Entity, &Self, &TActionKeyButton)>,
		agents: Query<Entity, With<TAgent>>,
		param: StaticSystemParam<TLoadout>,
	) where
		TAgent: Component,
		TActionKeyButton: Component + GetProperty<MouseOverrideActive>,
		TLoadout: for<'c> EntityContext<Skills, TContext<'c>: ReadSkills>,
	{
		set_color(commands, buttons, agents, param)
	}
}

fn set_color<TAgent, TActionKeyButton, TLoadout>(
	mut commands: Commands,
	buttons: Query<(Entity, &QuickbarPanel, &TActionKeyButton)>,
	agents: Query<Entity, With<TAgent>>,
	param: StaticSystemParam<TLoadout>,
) where
	TAgent: Component,
	TActionKeyButton: Component + GetProperty<MouseOverrideActive>,
	TLoadout: for<'c> EntityContext<Skills, TContext<'c>: ReadSkills>,
{
	for agent in &agents {
		let Some(ctx) = TLoadout::get_entity_context(&param, agent, Skills) else {
			continue;
		};

		for (entity, panel, action_button) in &buttons {
			let Ok(entity) = commands.get_entity(entity) else {
				continue;
			};
			let color = get_color_override(panel, action_button, &ctx);
			update_color_override(color, entity);
		}
	}
}

fn get_color_override<TActionKeyButton, TContext>(
	QuickbarPanel { key, .. }: &QuickbarPanel,
	action_button: &TActionKeyButton,
	ctx: &TContext,
) -> Option<ColorConfig>
where
	TActionKeyButton: Component + GetProperty<MouseOverrideActive>,
	TContext: ReadSkills,
{
	let skill = ctx.get_skill(*key)?;
	let activate_on_mouse_left = || action_button.dyn_property::<MouseOverrideActive>();

	match skill.dyn_property::<SkillExecution>() {
		SkillExecution::Active => Some(QuickbarPanel::ACTIVE_COLORS),
		SkillExecution::Queued => Some(QuickbarPanel::QUEUED_COLORS),
		SkillExecution::None if activate_on_mouse_left() => {
			Some(QuickbarPanel::PANEL_COLORS.pressed)
		}
		SkillExecution::None => None,
	}
}

fn update_color_override(color: Option<ColorConfig>, mut entity: EntityCommands) {
	match color {
		Some(ColorConfig { background, text }) => {
			entity.try_insert((
				ColorOverride,
				BackgroundColor::from(background),
				DispatchTextColor::from(text),
			));
		}
		None => {
			entity.remove::<ColorOverride>();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{ColorOverride, dispatch_text_color::DispatchTextColor};
	use bevy::state::app::StatesPlugin;
	use common::{
		tools::action_key::slot::PlayerSlot,
		traits::{
			handles_loadout::{
				LoadoutKey,
				loadout::{SkillIcon, SkillToken},
			},
			handles_localization::Token,
		},
	};
	use std::{collections::HashMap, sync::LazyLock};
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _ActionKeyButton {
		mouse_overridden: bool,
	}

	impl GetProperty<MouseOverrideActive> for _ActionKeyButton {
		fn get_property(&self) -> bool {
			self.mouse_overridden
		}
	}

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
	struct _Skill(SkillExecution);

	const IMAGE: Handle<Image> = Handle::Weak(AssetId::Uuid {
		uuid: AssetId::<Image>::DEFAULT_UUID,
	});

	impl GetProperty<SkillIcon> for _Skill {
		fn get_property(&self) -> &'_ Handle<Image> {
			&IMAGE
		}
	}

	static TOKEN: LazyLock<Token> = LazyLock::new(|| Token::from("my skill"));

	impl GetProperty<SkillToken> for _Skill {
		fn get_property(&self) -> &'_ Token {
			&TOKEN
		}
	}

	impl GetProperty<SkillExecution> for _Skill {
		fn get_property(&self) -> SkillExecution {
			self.0
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			set_color::<_Agent, _ActionKeyButton, Query<Ref<_Skills>>>,
		);
		app.add_plugins(StatesPlugin);

		app
	}

	#[test]
	fn set_to_active_when_matching_skill_active() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Skills(HashMap::from([(
				LoadoutKey::from(PlayerSlot::LOWER_L),
				_Skill(SkillExecution::Active),
			)])),
		));
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
				_ActionKeyButton {
					mouse_overridden: false,
				},
				QuickbarPanel::from(PlayerSlot::LOWER_L),
			))
			.id();

		app.update();

		let panel = app.world().entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();
		let text = panel.get::<DispatchTextColor>();
		assert_eq!(
			(
				QuickbarPanel::ACTIVE_COLORS.background,
				true,
				Some(&DispatchTextColor::from(QuickbarPanel::ACTIVE_COLORS.text))
			),
			(color.0, panel.contains::<ColorOverride>(), text)
		)
	}

	#[test]
	fn no_override_when_no_matching_skill_active() {
		let mut app = setup();
		app.world_mut().spawn((_Agent, _Skills(HashMap::from([]))));
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
				_ActionKeyButton {
					mouse_overridden: false,
				},
				QuickbarPanel::from(PlayerSlot::LOWER_L),
			))
			.id();

		app.update();

		let panel = app.world().entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();
		assert_eq!(
			(Color::NONE, false),
			(color.0, panel.contains::<ColorOverride>())
		);
	}

	#[test]
	fn no_override_when_skill_not_active() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Skills(HashMap::from([(
				LoadoutKey::from(PlayerSlot::LOWER_L),
				_Skill(SkillExecution::None),
			)])),
		));
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
				_ActionKeyButton {
					mouse_overridden: false,
				},
				QuickbarPanel::from(PlayerSlot::LOWER_L),
			))
			.id();

		app.update();

		let panel = app.world().entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();
		assert_eq!(
			(Color::NONE, false),
			(color.0, panel.contains::<ColorOverride>())
		);
	}

	#[test]
	fn set_to_pressed_when_matching_key_primed() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Skills(HashMap::from([(
				LoadoutKey::from(PlayerSlot::LOWER_L),
				_Skill(SkillExecution::None),
			)])),
		));
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
				_ActionKeyButton {
					mouse_overridden: true,
				},
				QuickbarPanel::from(PlayerSlot::LOWER_L),
			))
			.id();

		app.update();

		let panel = app.world().entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();
		let text = panel.get::<DispatchTextColor>();
		assert_eq!(
			(
				QuickbarPanel::PANEL_COLORS.pressed.background,
				true,
				Some(&DispatchTextColor::from(
					QuickbarPanel::PANEL_COLORS.pressed.text
				))
			),
			(color.0, panel.contains::<ColorOverride>(), text)
		)
	}

	#[test]
	fn set_to_queued_when_matching_with_queued_skill() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent,
			_Skills(HashMap::from([(
				LoadoutKey::from(PlayerSlot::LOWER_L),
				_Skill(SkillExecution::Queued),
			)])),
		));
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
				_ActionKeyButton {
					mouse_overridden: false,
				},
				QuickbarPanel::from(PlayerSlot::LOWER_L),
			))
			.id();

		app.update();

		let panel = app.world().entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();
		let text = panel.get::<DispatchTextColor>();
		assert_eq!(
			(
				QuickbarPanel::QUEUED_COLORS.background,
				true,
				Some(&DispatchTextColor::from(QuickbarPanel::QUEUED_COLORS.text))
			),
			(color.0, panel.contains::<ColorOverride>(), text)
		)
	}

	#[test]
	fn do_nothing_if_slots_has_no_agent() {
		let mut app = setup();
		app.world_mut().spawn(_Skills(HashMap::from([(
			LoadoutKey::from(PlayerSlot::LOWER_L),
			_Skill(SkillExecution::Active),
		)])));
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
				_ActionKeyButton {
					mouse_overridden: false,
				},
				QuickbarPanel::from(PlayerSlot::LOWER_L),
			))
			.id();

		app.update();

		let panel = app.world().entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();
		let text = panel.get::<DispatchTextColor>();
		assert_eq!(
			(Color::NONE, false, None),
			(color.0, panel.contains::<ColorOverride>(), text)
		)
	}
}
