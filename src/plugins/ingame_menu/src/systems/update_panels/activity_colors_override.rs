use crate::{
	components::ColorOverride,
	traits::{
		colors::{HasActiveColor, HasPanelColors, HasQueuedColor},
		get::Get,
	},
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
use common::components::Player;
use skills::{
	components::{Queue, SlotKey, Track},
	resources::SlotMap,
	skill::{Active, Skill},
	states::MouseContext,
};

type ActiveSkill<'a> = Option<&'a Track<Skill<Active>>>;

pub fn panel_activity_colors_override<
	TPanel: HasActiveColor + HasPanelColors + HasQueuedColor + Get<(), SlotKey> + Component,
>(
	mut commands: Commands,
	mut buttons: Query<(Entity, &mut BackgroundColor, &TPanel)>,
	player: Query<(ActiveSkill, &Queue), With<Player>>,
	slot_map: Res<SlotMap<KeyCode>>,
	mouse_context: Res<State<MouseContext>>,
) {
	let player_slots = &player.get_single().map(|(track, queue)| {
		(
			track.map(|t| t.value.data.0),
			queue.0.iter().map(|s| s.data.0).collect::<Vec<_>>(),
		)
	});
	let primed_slots = match mouse_context.get() {
		MouseContext::Primed(key) => slot_map.slots.get(key),
		_ => None,
	};

	for (entity, background_color, panel) in &mut buttons {
		let Some(entity) = commands.get_entity(entity) else {
			continue;
		};
		update_color_override(
			get_color::<TPanel>(player_slots, primed_slots, &panel.get(())),
			entity,
			background_color,
		);
	}
}

fn get_color<TPanel: HasActiveColor + HasPanelColors + HasQueuedColor>(
	player_slots: &Result<(Option<SlotKey>, Vec<SlotKey>), QuerySingleError>,
	primed_slot: Option<&SlotKey>,
	panel_key: &SlotKey,
) -> Option<bevy::prelude::Color> {
	match (player_slots, primed_slot) {
		(Ok((Some(active), _)), _) if active == panel_key => Some(TPanel::ACTIVE_COLOR),
		(Ok((_, queued)), _) if queued.contains(panel_key) => Some(TPanel::QUEUED_COLOR),
		(_, Some(primed)) if primed == panel_key => Some(TPanel::PANEL_COLORS.pressed),
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
	use crate::traits::colors::PanelColors;

	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::{bundle::Bundle, schedule::NextState},
		input::keyboard::KeyCode,
		render::color::Color,
	};
	use common::components::Side;
	use skills::skill::Queued;

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

	impl Get<(), SlotKey> for _Panel {
		fn get(&self, _: ()) -> SlotKey {
			self.0
		}
	}

	fn setup<TBundle: Bundle, const N: usize>(
		slot_key: Option<SlotKey>,
		bundle: TBundle,
		slot_map: [(KeyCode, SlotKey, &'static str); N],
	) -> (App, Entity, Entity) {
		let mut app = App::new();

		app.add_systems(Update, panel_activity_colors_override::<_Panel>);
		app.init_state::<MouseContext>();
		app.insert_resource(SlotMap::<KeyCode>::new(slot_map));
		let player = app.world.spawn((Player, Queue::default())).id();
		let panel = app.world.spawn(bundle).id();

		if let Some(slot_key) = slot_key {
			let mut player = app.world.entity_mut(player);
			player.insert(Track::new(Skill::default().with(Active(slot_key))));
		}

		(app, panel, player)
	}

	#[test]
	fn set_to_active_when_matching_skill_active() {
		let bundle = (
			BackgroundColor::from(Color::NONE),
			_Panel(SlotKey::Hand(Side::Main)),
		);
		let (mut app, panel, _) = setup(Some(SlotKey::Hand(Side::Main)), bundle, []);

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
		let (mut app, panel, _) = setup(Some(SlotKey::SkillSpawn), bundle, []);

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
		let (mut app, panel, _) = setup(None, bundle, []);

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
		let (mut app, panel, _) = setup(
			None,
			bundle,
			[(KeyCode::KeyQ, SlotKey::Hand(Side::Main), "")],
		);

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
		let (mut app, panel, player) = setup(None, bundle, []);

		let mut player = app.world.entity_mut(player);
		let mut queue = player.get_mut::<Queue>().unwrap();
		queue
			.0
			.push_back(Skill::default().with(Queued(SlotKey::Hand(Side::Main))));

		app.update();

		let panel = app.world.entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();

		assert_eq!(
			(_Panel::QUEUED_COLOR, true),
			(color.0, panel.contains::<ColorOverride>())
		)
	}
}
