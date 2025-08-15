use crate::{
	components::{ColorOverride, dispatch_text_color::DispatchTextColor},
	traits::colors::{ColorConfig, HasActiveColor, HasPanelColors, HasQueuedColor},
};
use bevy::prelude::*;
use common::{
	components::ui_input_primer::IsPrimed,
	tools::{
		action_key::{slot::PlayerSlot, user_input::UserInput},
		skill_execution::SkillExecution,
	},
	traits::{
		accessors::get::{Getter, RefInto},
		handles_loadout_menu::GetItem,
		inspect_able::{InspectAble, InspectField},
		key_mappings::GetInput,
	},
};

pub fn panel_activity_colors_override<TMap, TPanel, TPrimer, TContainer>(
	mut commands: Commands,
	mut buttons: Query<(Entity, &TPanel, &TPrimer)>,
	map: Res<TMap>,
	container: Res<TContainer>,
) where
	TMap: Resource + GetInput<PlayerSlot, TInput = UserInput>,
	TContainer: Resource + GetItem<PlayerSlot>,
	TContainer::TItem: InspectAble<SkillExecution>,
	TPanel: HasActiveColor
		+ HasPanelColors
		+ HasQueuedColor
		+ for<'a> RefInto<'a, PlayerSlot>
		+ Component,
	TPrimer: Component,
	for<'a> &'a TPrimer: Into<IsPrimed> + Into<UserInput>,
{
	for (entity, panel, primer) in &mut buttons {
		let Ok(entity) = commands.get_entity(entity) else {
			continue;
		};
		let color = get_color_override(container.as_ref(), map.as_ref(), panel, primer);
		update_color_override(color, entity);
	}
}

fn get_color_override<TContainer, TMap, TPanel, TPrimer>(
	container: &TContainer,
	map: &TMap,
	panel: &TPanel,
	primer: &TPrimer,
) -> Option<ColorConfig>
where
	TPanel: HasActiveColor + HasPanelColors + HasQueuedColor + for<'a> RefInto<'a, PlayerSlot>,
	TContainer: GetItem<PlayerSlot>,
	TContainer::TItem: InspectAble<SkillExecution>,
	TMap: GetInput<PlayerSlot, TInput = UserInput>,
	for<'a> &'a TPrimer: Into<IsPrimed> + Into<UserInput>,
{
	let panel_key = panel.get::<PlayerSlot>();
	let state = container
		.get_item(panel_key)
		.map(SkillExecution::inspect_field)
		.copied()
		.unwrap_or_default();

	if state == SkillExecution::Active {
		return Some(TPanel::ACTIVE_COLORS);
	}

	if state == SkillExecution::Queued {
		return Some(TPanel::QUEUED_COLORS);
	}

	let IsPrimed(primer_is_primed) = primer.into();
	let primer_key: TMap::TInput = primer.into();
	if primer_is_primed && map.get_input(panel_key) == primer_key {
		return Some(TPanel::PANEL_COLORS.pressed);
	}

	None
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
	use crate::{
		components::dispatch_text_color::DispatchTextColor,
		traits::colors::{ColorConfig, PanelColors},
	};
	use bevy::state::app::StatesPlugin;
	use common::{
		components::ui_input_primer::IsPrimed,
		tools::action_key::{slot::Side, user_input::UserInput},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashMap;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component)]
	struct _Primer {
		key: UserInput,
		is_primed: IsPrimed,
	}

	impl From<&_Primer> for UserInput {
		fn from(_Primer { key, .. }: &_Primer) -> Self {
			*key
		}
	}

	impl From<&_Primer> for IsPrimed {
		fn from(_Primer { is_primed, .. }: &_Primer) -> Self {
			*is_primed
		}
	}

	#[derive(Component)]
	struct _Panel(pub PlayerSlot);

	impl HasActiveColor for _Panel {
		const ACTIVE_COLORS: ColorConfig = ColorConfig {
			background: Color::srgb(0.1, 0.2, 0.3),
			text: Color::srgb(0.1, 0.2, 0.7),
		};
	}

	impl HasQueuedColor for _Panel {
		const QUEUED_COLORS: ColorConfig = ColorConfig {
			background: Color::srgb(0.3, 0.2, 0.1),
			text: Color::srgb(0.3, 0.2, 0.7),
		};
	}

	impl HasPanelColors for _Panel {
		const PANEL_COLORS: PanelColors = PanelColors {
			disabled: ColorConfig::NO_COLORS,
			pressed: ColorConfig {
				background: Color::srgb(0.1, 1., 0.1),
				text: Color::srgb(1., 0.5, 0.25),
			},
			hovered: ColorConfig::NO_COLORS,
			empty: ColorConfig::NO_COLORS,
			filled: ColorConfig::NO_COLORS,
		};
	}

	impl From<&_Panel> for PlayerSlot {
		fn from(_Panel(slot): &_Panel) -> Self {
			*slot
		}
	}

	#[derive(Resource, NestedMocks)]
	struct _Map {
		mock: Mock_Map,
	}

	impl Default for _Map {
		fn default() -> Self {
			Self::new().with_mock(|mock| {
				mock.expect_get_input()
					.return_const(UserInput::from(KeyCode::KeyA));
			})
		}
	}

	#[automock]
	impl GetInput<PlayerSlot> for _Map {
		type TInput = UserInput;

		fn get_input(&self, value: PlayerSlot) -> UserInput {
			self.mock.get_input(value)
		}
	}

	#[derive(Resource)]
	struct _Cache(HashMap<PlayerSlot, _Item>);

	impl GetItem<PlayerSlot> for _Cache {
		type TItem = _Item;

		fn get_item(&self, key: PlayerSlot) -> Option<&_Item> {
			self.0.get(&key)
		}
	}

	impl<const N: usize> From<[(PlayerSlot, _Item); N]> for _Cache {
		fn from(value: [(PlayerSlot, _Item); N]) -> Self {
			Self(HashMap::from(value))
		}
	}

	struct _Item(SkillExecution);

	impl InspectAble<SkillExecution> for _Item {
		fn get_inspect_able_field(&self) -> &SkillExecution {
			&self.0
		}
	}

	fn setup(key_map: _Map, cache: _Cache) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			panel_activity_colors_override::<_Map, _Panel, _Primer, _Cache>,
		);
		app.add_plugins(StatesPlugin);
		app.insert_resource(key_map);
		app.insert_resource(cache);

		app
	}

	#[test]
	fn set_to_active_when_matching_skill_active() {
		let mut app = setup(
			_Map::default(),
			_Cache::from([(PlayerSlot::Lower(Side::Left), _Item(SkillExecution::Active))]),
		);
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
				_Primer {
					key: UserInput::from(KeyCode::KeyA),
					is_primed: IsPrimed(false),
				},
				_Panel(PlayerSlot::Lower(Side::Left)),
			))
			.id();

		app.update();

		let panel = app.world().entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();
		let text = panel.get::<DispatchTextColor>();
		assert_eq!(
			(
				_Panel::ACTIVE_COLORS.background,
				true,
				Some(&DispatchTextColor::from(_Panel::ACTIVE_COLORS.text))
			),
			(color.0, panel.contains::<ColorOverride>(), text)
		)
	}

	#[test]
	fn no_override_when_no_matching_skill_active() {
		let mut app = setup(_Map::default(), _Cache::from([]));
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
				_Primer {
					key: UserInput::from(KeyCode::KeyA),
					is_primed: IsPrimed(false),
				},
				_Panel(PlayerSlot::Lower(Side::Left)),
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
	fn no_override_when_no_skill_not_active() {
		let mut app = setup(
			_Map::default(),
			_Cache::from([(PlayerSlot::Lower(Side::Left), _Item(SkillExecution::None))]),
		);
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
				_Primer {
					key: UserInput::from(KeyCode::KeyA),
					is_primed: IsPrimed(false),
				},
				_Panel(PlayerSlot::Lower(Side::Left)),
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
		let mut app = setup(
			_Map::new().with_mock(|mock| {
				mock.expect_get_input()
					.times(1)
					.with(eq(PlayerSlot::Lower(Side::Left)))
					.return_const(UserInput::from(KeyCode::KeyQ));
			}),
			_Cache::from([]),
		);
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
				_Primer {
					key: UserInput::from(KeyCode::KeyQ),
					is_primed: IsPrimed(true),
				},
				_Panel(PlayerSlot::Lower(Side::Left)),
			))
			.id();

		app.update();

		let panel = app.world().entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();
		let text = panel.get::<DispatchTextColor>();
		assert_eq!(
			(
				_Panel::PANEL_COLORS.pressed.background,
				true,
				Some(&DispatchTextColor::from(_Panel::PANEL_COLORS.pressed.text))
			),
			(color.0, panel.contains::<ColorOverride>(), text)
		)
	}

	#[test]
	fn set_to_queued_when_matching_with_queued_skill() {
		let mut app = setup(
			_Map::default(),
			_Cache::from([(PlayerSlot::Lower(Side::Left), _Item(SkillExecution::Queued))]),
		);
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
				_Primer {
					key: UserInput::from(KeyCode::KeyA),
					is_primed: IsPrimed(false),
				},
				_Panel(PlayerSlot::Lower(Side::Left)),
			))
			.id();

		app.update();

		let panel = app.world().entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();
		let text = panel.get::<DispatchTextColor>();
		assert_eq!(
			(
				_Panel::QUEUED_COLORS.background,
				true,
				Some(&DispatchTextColor::from(_Panel::QUEUED_COLORS.text))
			),
			(color.0, panel.contains::<ColorOverride>(), text)
		)
	}
}
