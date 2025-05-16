use crate::{
	components::ColorOverride,
	traits::colors::{HasActiveColor, HasPanelColors, HasQueuedColor},
};
use bevy::prelude::*;
use common::{
	components::ui_input_primer::IsPrimed,
	tools::{
		action_key::{slot::SlotKey, user_input::UserInput},
		skill_execution::SkillExecution,
	},
	traits::{
		accessors::get::{GetFieldRef, GetterRef},
		handles_loadout_menu::GetItem,
		inspect_able::{InspectAble, InspectField},
		key_mappings::GetInput,
	},
};

pub fn panel_activity_colors_override<TMap, TPanel, TPrimer, TContainer>(
	mut commands: Commands,
	mut buttons: Query<(Entity, &mut BackgroundColor, &TPanel, &TPrimer)>,
	map: Res<TMap>,
	container: Res<TContainer>,
) where
	TMap: Resource + GetInput<SlotKey, UserInput>,
	TContainer: Resource + GetItem<SlotKey>,
	TContainer::TItem: InspectAble<SkillExecution>,
	TPanel: HasActiveColor + HasPanelColors + HasQueuedColor + GetterRef<SlotKey> + Component,
	TPrimer: Component,
	for<'a> &'a TPrimer: Into<IsPrimed> + Into<UserInput>,
{
	for (entity, background_color, panel, primer) in &mut buttons {
		let Some(entity) = commands.get_entity(entity) else {
			continue;
		};
		let color = get_color_override(container.as_ref(), map.as_ref(), panel, primer);
		update_color_override(color, entity, background_color);
	}
}

fn get_color_override<TContainer, TMap, TPanel, TPrimer>(
	container: &TContainer,
	map: &TMap,
	panel: &TPanel,
	primer: &TPrimer,
) -> Option<Color>
where
	TPanel: HasActiveColor + HasPanelColors + HasQueuedColor + GetterRef<SlotKey>,
	TContainer: GetItem<SlotKey>,
	TContainer::TItem: InspectAble<SkillExecution>,
	TMap: GetInput<SlotKey, UserInput>,
	for<'a> &'a TPrimer: Into<IsPrimed> + Into<UserInput>,
{
	let panel_key = SlotKey::get_field_ref(panel);
	let state = container
		.get_item(*panel_key)
		.map(SkillExecution::inspect_field)
		.copied()
		.unwrap_or_default();

	if state == SkillExecution::Active {
		return Some(TPanel::ACTIVE_COLOR);
	}

	if state == SkillExecution::Queued {
		return Some(TPanel::QUEUED_COLOR);
	}

	let IsPrimed(primer_is_primed) = primer.into();
	let primer_key: UserInput = primer.into();
	if primer_is_primed && map.get_input(*panel_key) == primer_key {
		return Some(TPanel::PANEL_COLORS.pressed);
	}

	None
}

fn update_color_override(
	color: Option<Color>,
	mut entity: EntityCommands,
	mut background_color: Mut<BackgroundColor>,
) {
	match color {
		Some(color) => {
			entity.try_insert(ColorOverride);
			*background_color = color.into();
		}
		None => {
			entity.remove::<ColorOverride>();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::colors::PanelColors;
	use bevy::state::app::StatesPlugin;
	use common::{
		components::ui_input_primer::IsPrimed,
		test_tools::utils::SingleThreadedApp,
		tools::action_key::slot::Side,
		traits::nested_mock::NestedMocks,
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashMap;

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
	struct _Panel(pub SlotKey);

	impl HasActiveColor for _Panel {
		const ACTIVE_COLOR: Color = Color::srgb(0.1, 0.2, 0.3);
	}

	impl HasQueuedColor for _Panel {
		const QUEUED_COLOR: Color = Color::srgb(0.3, 0.2, 0.1);
	}

	impl HasPanelColors for _Panel {
		const PANEL_COLORS: PanelColors = PanelColors {
			pressed: Color::srgb(0.1, 1., 0.1),
			hovered: Color::NONE,
			empty: Color::NONE,
			filled: Color::NONE,
			text: Color::NONE,
		};
	}

	impl GetterRef<SlotKey> for _Panel {
		fn get(&self) -> &SlotKey {
			&self.0
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
	impl GetInput<SlotKey, UserInput> for _Map {
		fn get_input(&self, value: SlotKey) -> UserInput {
			self.mock.get_input(value)
		}
	}

	#[derive(Resource)]
	struct _Cache(HashMap<SlotKey, _Item>);

	impl GetItem<SlotKey> for _Cache {
		type TItem = _Item;

		fn get_item(&self, key: SlotKey) -> Option<&_Item> {
			self.0.get(&key)
		}
	}

	impl<const N: usize> From<[(SlotKey, _Item); N]> for _Cache {
		fn from(value: [(SlotKey, _Item); N]) -> Self {
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
			_Cache::from([(
				SlotKey::BottomHand(Side::Right),
				_Item(SkillExecution::Active),
			)]),
		);
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
				_Primer {
					key: UserInput::from(KeyCode::KeyA),
					is_primed: IsPrimed(false),
				},
				_Panel(SlotKey::BottomHand(Side::Right)),
			))
			.id();

		app.update();

		let panel = app.world().entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();
		assert_eq!(
			(_Panel::ACTIVE_COLOR, true),
			(color.0, panel.contains::<ColorOverride>())
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
				_Panel(SlotKey::BottomHand(Side::Right)),
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
			_Cache::from([(
				SlotKey::BottomHand(Side::Right),
				_Item(SkillExecution::None),
			)]),
		);
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
				_Primer {
					key: UserInput::from(KeyCode::KeyA),
					is_primed: IsPrimed(false),
				},
				_Panel(SlotKey::BottomHand(Side::Right)),
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
					.with(eq(SlotKey::BottomHand(Side::Right)))
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
				_Panel(SlotKey::BottomHand(Side::Right)),
			))
			.id();

		app.update();

		let panel = app.world().entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();
		assert_eq!(
			(_Panel::PANEL_COLORS.pressed, true),
			(color.0, panel.contains::<ColorOverride>())
		)
	}

	#[test]
	fn set_to_queued_when_matching_with_queued_skill() {
		let mut app = setup(
			_Map::default(),
			_Cache::from([(
				SlotKey::BottomHand(Side::Right),
				_Item(SkillExecution::Queued),
			)]),
		);
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
				_Primer {
					key: UserInput::from(KeyCode::KeyA),
					is_primed: IsPrimed(false),
				},
				_Panel(SlotKey::BottomHand(Side::Right)),
			))
			.id();

		app.update();

		let panel = app.world().entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();
		assert_eq!(
			(_Panel::QUEUED_COLOR, true),
			(color.0, panel.contains::<ColorOverride>())
		)
	}
}
