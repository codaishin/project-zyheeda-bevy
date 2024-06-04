use crate::{
	components::ColorOverride,
	traits::colors::{HasActiveColor, HasPanelColors, HasQueuedColor},
};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		query::{QuerySingleError, With},
		schedule::State,
		system::{Commands, EntityCommands, Query, Res},
		world::Mut,
	},
	input::keyboard::KeyCode,
	render::color::Color,
	ui::BackgroundColor,
};
use common::{
	components::Player,
	states::MouseContext,
	traits::{get::GetStatic, iterate::Iterate},
};
use skills::{
	items::SlotKey,
	resources::SlotMap,
	skills::{Queued, Skill},
};

pub fn panel_activity_colors_override<
	TQueue: Component + Iterate<Skill<Queued>>,
	TPanel: HasActiveColor + HasPanelColors + HasQueuedColor + GetStatic<SlotKey> + Component,
>(
	mut commands: Commands,
	mut buttons: Query<(Entity, &mut BackgroundColor, &TPanel)>,
	player: Query<&TQueue, With<Player>>,
	slot_map: Res<SlotMap<KeyCode>>,
	mouse_context: Res<State<MouseContext>>,
) {
	let player_slots = &player
		.get_single()
		.map(|queue| queue.iterate().map(|s| s.data.slot_key).collect::<Vec<_>>());
	let primed_slots = match mouse_context.get() {
		MouseContext::Primed(key) => slot_map.slots.get(key),
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
	primed_slot: Option<&SlotKey>,
	panel_key: &SlotKey,
) -> Option<Color> {
	let Ok(player_slots) = player_slots else {
		return None;
	};

	let mut iter = player_slots.iterate();

	match (iter.next(), iter.collect::<Vec<_>>(), primed_slot) {
		(Some(active), _, _) if active == panel_key => Some(TPanel::ACTIVE_COLOR),
		(_, queued, _) if queued.contains(&panel_key) => Some(TPanel::QUEUED_COLOR),
		(_, _, Some(primed)) if primed == panel_key => Some(TPanel::PANEL_COLORS.pressed),
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
	use bevy::{
		app::{App, Update},
		ecs::{bundle::Bundle, schedule::NextState},
		input::keyboard::KeyCode,
		render::color::Color,
		utils::default,
	};
	use common::components::Side;
	use skills::skills::Queued;

	#[derive(Component)]
	struct _Panel(pub SlotKey);

	impl HasActiveColor for _Panel {
		const ACTIVE_COLOR: Color = Color::BEIGE;
	}

	impl HasQueuedColor for _Panel {
		const QUEUED_COLOR: Color = Color::ANTIQUE_WHITE;
	}

	impl HasPanelColors for _Panel {
		const PANEL_COLORS: PanelColors = PanelColors {
			pressed: Color::DARK_GREEN,
			hovered: Color::NONE,
			empty: Color::NONE,
			filled: Color::NONE,
			text: Color::NONE,
		};
	}

	impl GetStatic<SlotKey> for _Panel {
		fn get(&self) -> &SlotKey {
			&self.0
		}
	}

	#[derive(Component, Default)]
	struct _Queue {
		queued: Vec<Skill<Queued>>,
	}

	impl Iterate<Skill<Queued>> for _Queue {
		fn iterate<'a>(&'a self) -> impl DoubleEndedIterator<Item = &'a Skill<Queued>>
		where
			Skill<Queued>: 'a,
		{
			self.queued.iterate()
		}
	}

	fn setup<TBundle: Bundle, const N: usize>(
		bundle: TBundle,
		slot_map: [(KeyCode, SlotKey, &'static str); N],
	) -> (App, Entity) {
		let mut app = App::new();

		app.add_systems(Update, panel_activity_colors_override::<_Queue, _Panel>);
		app.init_state::<MouseContext>();
		app.insert_resource(SlotMap::<KeyCode>::new(slot_map));
		let panel = app.world.spawn(bundle).id();

		(app, panel)
	}

	#[test]
	fn set_to_active_when_matching_skill_active() {
		let bundle = (
			BackgroundColor::from(Color::NONE),
			_Panel(SlotKey::Hand(Side::Main)),
		);
		let (mut app, panel) = setup(bundle, []);

		app.world.spawn((
			Player,
			_Queue {
				queued: vec![Skill {
					data: Queued {
						slot_key: SlotKey::Hand(Side::Main),
						..default()
					},
					..default()
				}],
			},
		));

		app.update();

		let panel = app.world.entity(panel);
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
			_Panel(SlotKey::Hand(Side::Main)),
			ColorOverride,
		);
		let (mut app, panel) = setup(bundle, []);

		app.world.spawn((
			Player,
			_Queue {
				queued: vec![Skill {
					data: Queued {
						slot_key: SlotKey::Hand(Side::Off),
						..default()
					},
					..default()
				}],
			},
		));

		app.update();

		let panel = app.world.entity(panel);
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
			_Panel(SlotKey::Hand(Side::Main)),
			ColorOverride,
		);
		let (mut app, panel) = setup(bundle, []);

		app.world.spawn((Player, _Queue { queued: vec![] }));

		app.update();

		let panel = app.world.entity(panel);
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
			_Panel(SlotKey::Hand(Side::Main)),
		);
		let (mut app, panel) = setup(bundle, [(KeyCode::KeyQ, SlotKey::Hand(Side::Main), "")]);

		app.world.spawn((Player, _Queue { queued: vec![] }));

		app.world
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Primed(KeyCode::KeyQ));

		app.update();
		app.update();

		let panel = app.world.entity(panel);
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
			_Panel(SlotKey::Hand(Side::Main)),
		);
		let (mut app, panel) = setup(bundle, []);

		app.world.spawn((
			Player,
			_Queue {
				queued: vec![
					Skill {
						data: Queued {
							slot_key: SlotKey::Hand(Side::Off),
							..default()
						},
						..default()
					},
					Skill {
						data: Queued {
							slot_key: SlotKey::Hand(Side::Main),
							..default()
						},
						..default()
					},
				],
			},
		));

		app.update();

		let panel = app.world.entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();

		assert_eq!(
			(_Panel::QUEUED_COLOR, true),
			(color.0, panel.contains::<ColorOverride>())
		)
	}
}
