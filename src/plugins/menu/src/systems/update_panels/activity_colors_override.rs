use crate::{
	components::ColorOverride,
	traits::colors::{HasActiveColor, HasPanelColors, HasQueuedColor},
};
use bevy::{ecs::query::QuerySingleError, prelude::*};
use common::{
	states::mouse_context::MouseContext,
	tools::slot_key::SlotKey,
	traits::{
		accessors::get::{GetField, Getter, GetterRef},
		handles_equipment::IterateQueue,
		iterate::Iterate,
		map_value::TryMapBackwards,
	},
};

pub fn panel_activity_colors_override<TPlayer, TMap, TQueue, TPanel>(
	mut commands: Commands,
	mut buttons: Query<(Entity, &mut BackgroundColor, &TPanel)>,
	player: Query<&TQueue, With<TPlayer>>,
	key_map: Res<TMap>,
	mouse_context: Res<State<MouseContext>>,
) where
	TPlayer: Component,
	TMap: Resource + TryMapBackwards<KeyCode, SlotKey>,
	TQueue: Component + IterateQueue,
	TQueue::TItem: Getter<SlotKey>,
	TPanel: HasActiveColor + HasPanelColors + HasQueuedColor + GetterRef<SlotKey> + Component,
{
	let player_slots = &player
		.get_single()
		.map(|queue| queue.iterate().map(SlotKey::get_field).collect::<Vec<_>>());
	let primed_slots = match mouse_context.get() {
		MouseContext::Primed(key) => key_map.try_map_backwards(*key),
		_ => None,
	};

	for (entity, background_color, panel) in &mut buttons {
		let Some(entity) = commands.get_entity(entity) else {
			continue;
		};
		update_color_override(
			get_color::<TPanel>(player_slots, primed_slots, panel.get()),
			entity,
			background_color,
		);
	}
}

fn get_color<TPanel: HasActiveColor + HasPanelColors + HasQueuedColor>(
	player_slots: &Result<Vec<SlotKey>, QuerySingleError>,
	primed_slot: Option<SlotKey>,
	panel_key: &SlotKey,
) -> Option<Color> {
	let Ok(player_slots) = player_slots else {
		return None;
	};

	let mut iter = player_slots.iterate();

	match (iter.next(), iter.collect::<Vec<_>>(), primed_slot) {
		(Some(active), _, _) if active == panel_key => Some(TPanel::ACTIVE_COLOR),
		(_, queued, _) if queued.contains(&panel_key) => Some(TPanel::QUEUED_COLOR),
		(_, _, Some(primed)) if &primed == panel_key => Some(TPanel::PANEL_COLORS.pressed),
		_ => None,
	}
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
	use common::tools::slot_key::Side;

	#[derive(Component)]
	struct _Player;

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

	struct _QueuedSkill(SlotKey);

	impl Getter<SlotKey> for _QueuedSkill {
		fn get(&self) -> SlotKey {
			self.0
		}
	}

	#[derive(Component, Default)]
	struct _Queue {
		queued: Vec<_QueuedSkill>,
	}

	impl IterateQueue for _Queue {
		type TItem = _QueuedSkill;

		fn iterate(&self) -> impl Iterator<Item = &Self::TItem> {
			self.queued.iterate()
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

	fn setup<TBundle: Bundle>(bundle: TBundle, key_map: _Map) -> (App, Entity) {
		let mut app = App::new();

		app.add_systems(
			Update,
			panel_activity_colors_override::<_Player, _Map, _Queue, _Panel>,
		);
		app.add_plugins(StatesPlugin);
		app.init_state::<MouseContext>();
		app.insert_resource(key_map);
		let panel = app.world_mut().spawn(bundle).id();

		(app, panel)
	}

	#[test]
	fn set_to_active_when_matching_skill_active() {
		let bundle = (
			BackgroundColor::from(Color::NONE),
			_Panel(SlotKey::BottomHand(Side::Right)),
		);
		let (mut app, panel) = setup(bundle, _Map::None);

		app.world_mut().spawn((
			_Player,
			_Queue {
				queued: vec![_QueuedSkill(SlotKey::BottomHand(Side::Right))],
			},
		));

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
		let bundle = (
			BackgroundColor::from(Color::NONE),
			_Panel(SlotKey::BottomHand(Side::Right)),
			ColorOverride,
		);
		let (mut app, panel) = setup(bundle, _Map::None);

		app.world_mut().spawn((
			_Player,
			_Queue {
				queued: vec![_QueuedSkill(SlotKey::BottomHand(Side::Left))],
			},
		));

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
		let bundle = (
			BackgroundColor::from(Color::NONE),
			_Panel(SlotKey::BottomHand(Side::Right)),
			ColorOverride,
		);
		let (mut app, panel) = setup(bundle, _Map::None);

		app.world_mut().spawn((_Player, _Queue { queued: vec![] }));

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
		let bundle = (
			BackgroundColor::from(Color::NONE),
			_Panel(SlotKey::BottomHand(Side::Right)),
		);
		let (mut app, panel) = setup(
			bundle,
			_Map::Map(KeyCode::KeyQ, SlotKey::BottomHand(Side::Right)),
		);

		app.world_mut().spawn((_Player, _Queue { queued: vec![] }));

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
		let bundle = (
			BackgroundColor::from(Color::NONE),
			_Panel(SlotKey::BottomHand(Side::Right)),
		);
		let (mut app, panel) = setup(bundle, _Map::None);

		app.world_mut().spawn((
			_Player,
			_Queue {
				queued: vec![
					_QueuedSkill(SlotKey::BottomHand(Side::Left)),
					_QueuedSkill(SlotKey::BottomHand(Side::Right)),
				],
			},
		));

		app.update();

		let panel = app.world().entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();

		assert_eq!(
			(_Panel::QUEUED_COLOR, true),
			(color.0, panel.contains::<ColorOverride>())
		)
	}
}
