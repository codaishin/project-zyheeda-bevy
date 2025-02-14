use crate::{
	components::ColorOverride,
	traits::colors::{HasActiveColor, HasPanelColors, HasQueuedColor},
};
use bevy::prelude::*;
use common::{
	states::mouse_context::MouseContext,
	tools::slot_key::SlotKey,
	traits::{
		accessors::get::{GetFieldRef, GetterRef},
		handles_inventory_menu::{GetDescriptor, QuickbarDescriptor, SkillExecution},
		map_value::TryMapBackwards,
	},
};

pub fn panel_activity_colors_override<TMap, TPanel, TDescriptors>(
	mut commands: Commands,
	mut buttons: Query<(Entity, &mut BackgroundColor, &TPanel)>,
	key_map: Res<TMap>,
	descriptors: Res<TDescriptors>,
	mouse_context: Res<State<MouseContext>>,
) where
	TMap: Resource + TryMapBackwards<KeyCode, SlotKey>,
	TDescriptors: Resource + GetDescriptor<SlotKey, TItem = QuickbarDescriptor>,
	TPanel: HasActiveColor + HasPanelColors + HasQueuedColor + GetterRef<SlotKey> + Component,
{
	let primed_slot = match mouse_context.get() {
		MouseContext::Primed(key) => key_map.try_map_backwards(*key),
		_ => None,
	};

	for (entity, background_color, panel) in &mut buttons {
		let Some(entity) = commands.get_entity(entity) else {
			continue;
		};
		let color = get_color_override::<TPanel, TDescriptors>(
			&descriptors,
			primed_slot,
			SlotKey::get_field_ref(panel),
		);
		update_color_override(color, entity, background_color);
	}
}

fn get_color_override<TPanel, TDescriptors>(
	descriptors: &TDescriptors,
	primed_slot: Option<SlotKey>,
	panel_key: &SlotKey,
) -> Option<Color>
where
	TPanel: HasActiveColor + HasPanelColors + HasQueuedColor,
	TDescriptors: GetDescriptor<SlotKey, TItem = QuickbarDescriptor>,
{
	let state = descriptors
		.get_descriptor(*panel_key)
		.map(|descriptor| descriptor.execution)
		.unwrap_or_default();

	if state == SkillExecution::Active {
		return Some(TPanel::ACTIVE_COLOR);
	}

	if state == SkillExecution::Queued {
		return Some(TPanel::QUEUED_COLOR);
	}

	if &primed_slot? == panel_key {
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
	use common::{test_tools::utils::SingleThreadedApp, tools::slot_key::Side};
	use std::collections::HashMap;

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

	#[derive(Resource)]
	enum _Map {
		None,
		Map(KeyCode, SlotKey),
	}

	impl TryMapBackwards<KeyCode, SlotKey> for _Map {
		fn try_map_backwards(&self, value: KeyCode) -> Option<SlotKey> {
			match self {
				_Map::Map(key, slot) if key == &value => Some(*slot),
				_ => None,
			}
		}
	}

	#[derive(Resource)]
	struct _Cache(HashMap<SlotKey, QuickbarDescriptor>);

	impl GetDescriptor<SlotKey> for _Cache {
		type TItem = QuickbarDescriptor;

		fn get_descriptor(&self, key: SlotKey) -> Option<&QuickbarDescriptor> {
			self.0.get(&key)
		}
	}

	impl<const N: usize> From<[(SlotKey, QuickbarDescriptor); N]> for _Cache {
		fn from(value: [(SlotKey, QuickbarDescriptor); N]) -> Self {
			Self(HashMap::from(value))
		}
	}

	fn setup(key_map: _Map, cache: _Cache) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			panel_activity_colors_override::<_Map, _Panel, _Cache>,
		);
		app.add_plugins(StatesPlugin);
		app.init_state::<MouseContext>();
		app.insert_resource(key_map);
		app.insert_resource(cache);

		app
	}

	#[test]
	fn set_to_active_when_matching_skill_active() {
		let mut app = setup(
			_Map::None,
			_Cache::from([(
				SlotKey::BottomHand(Side::Right),
				QuickbarDescriptor {
					execution: SkillExecution::Active,
					..default()
				},
			)]),
		);
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
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
		let mut app = setup(_Map::None, _Cache::from([]));
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
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
			_Map::None,
			_Cache::from([(
				SlotKey::BottomHand(Side::Right),
				QuickbarDescriptor {
					execution: SkillExecution::None,
					..default()
				},
			)]),
		);
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
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
	fn set_to_pressed_when_matching_key_primed_in_mouse_context() {
		let mut app = setup(
			_Map::Map(KeyCode::KeyQ, SlotKey::BottomHand(Side::Right)),
			_Cache::from([]),
		);
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
				_Panel(SlotKey::BottomHand(Side::Right)),
			))
			.id();

		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Primed(KeyCode::KeyQ));
		app.update();
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
			_Map::None,
			_Cache::from([(
				SlotKey::BottomHand(Side::Right),
				QuickbarDescriptor {
					execution: SkillExecution::Queued,
					..default()
				},
			)]),
		);
		let panel = app
			.world_mut()
			.spawn((
				BackgroundColor::from(Color::NONE),
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
