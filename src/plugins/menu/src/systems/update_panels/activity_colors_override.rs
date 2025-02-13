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
		handles_inventory_menu::GetDescriptor,
		handles_quickbar::{ActiveSlotKey, QueuedSlotKey},
		map_value::TryMapBackwards,
	},
};

pub fn panel_activity_colors_override<TMap, TPanel, TActives, TQueued>(
	mut commands: Commands,
	mut buttons: Query<(Entity, &mut BackgroundColor, &TPanel)>,
	key_map: Res<TMap>,
	active_skills: Res<TActives>,
	queued_skills: Res<TQueued>,
	mouse_context: Res<State<MouseContext>>,
) where
	TMap: Resource + TryMapBackwards<KeyCode, SlotKey>,
	TActives: Resource + GetDescriptor<ActiveSlotKey>,
	TQueued: Resource + GetDescriptor<QueuedSlotKey>,
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
		let color = get_color::<TPanel, TActives, TQueued>(
			&active_skills,
			&queued_skills,
			primed_slot,
			SlotKey::get_field_ref(panel),
		);
		update_color_override(color, entity, background_color);
	}
}

fn get_color<TPanel, TActives, TQueued>(
	active: &TActives,
	queued: &TQueued,
	primed_slot: Option<SlotKey>,
	panel_key: &SlotKey,
) -> Option<Color>
where
	TPanel: HasActiveColor + HasPanelColors + HasQueuedColor,
	TActives: GetDescriptor<ActiveSlotKey>,
	TQueued: GetDescriptor<QueuedSlotKey>,
{
	if active.get_descriptor(ActiveSlotKey(*panel_key)).is_some() {
		return Some(TPanel::ACTIVE_COLOR);
	}

	if queued.get_descriptor(QueuedSlotKey(*panel_key)).is_some() {
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
	use std::collections::HashMap;

	use super::*;
	use crate::traits::colors::PanelColors;
	use bevy::state::app::StatesPlugin;
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::slot_key::Side,
		traits::handles_inventory_menu::Descriptor,
	};

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
	struct _Active(HashMap<ActiveSlotKey, Descriptor>);

	impl GetDescriptor<ActiveSlotKey> for _Active {
		fn get_descriptor(&self, key: ActiveSlotKey) -> Option<&Descriptor> {
			self.0.get(&key)
		}
	}

	impl<const N: usize> From<[SlotKey; N]> for _Active {
		fn from(value: [SlotKey; N]) -> Self {
			Self(HashMap::from(
				value.map(|key| (ActiveSlotKey(key), default())),
			))
		}
	}

	#[derive(Resource)]
	struct _Queued(HashMap<QueuedSlotKey, Descriptor>);

	impl GetDescriptor<QueuedSlotKey> for _Queued {
		fn get_descriptor(&self, key: QueuedSlotKey) -> Option<&Descriptor> {
			self.0.get(&key)
		}
	}

	impl<const N: usize> From<[SlotKey; N]> for _Queued {
		fn from(value: [SlotKey; N]) -> Self {
			Self(HashMap::from(
				value.map(|key| (QueuedSlotKey(key), default())),
			))
		}
	}

	fn setup(key_map: _Map, active: _Active, queued: _Queued) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			panel_activity_colors_override::<_Map, _Panel, _Active, _Queued>,
		);
		app.add_plugins(StatesPlugin);
		app.init_state::<MouseContext>();
		app.insert_resource(key_map);
		app.insert_resource(active);
		app.insert_resource(queued);

		app
	}

	#[test]
	fn set_to_active_when_matching_skill_active() {
		let mut app = setup(
			_Map::None,
			_Active::from([SlotKey::BottomHand(Side::Right)]),
			_Queued::from([]),
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
		let mut app = setup(
			_Map::None,
			_Active::from([SlotKey::TopHand(Side::Right)]),
			_Queued::from([]),
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
	fn no_override_when_no_skill_active() {
		let mut app = setup(_Map::None, _Active::from([]), _Queued::from([]));
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
			_Active::from([]),
			_Queued::from([]),
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
			_Active::from([]),
			_Queued::from([SlotKey::BottomHand(Side::Right)]),
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
