use crate::{
	components::{Player, SlotKey, Track},
	plugins::ingame_menu::{
		components::ColorOverride,
		traits::{
			colors::{HasActiveColor, HasPanelColors},
			get::Get,
		},
	},
	resources::SlotMap,
	skill::{Active, Skill},
	states::MouseContext,
};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		query::With,
		schedule::State,
		system::{Commands, Query, Res},
	},
	input::keyboard::KeyCode,
	ui::BackgroundColor,
};

pub fn panel_activity_colors_override<
	TPanel: HasActiveColor + HasPanelColors + Get<(), SlotKey> + Component,
>(
	mut commands: Commands,
	mut buttons: Query<(Entity, &mut BackgroundColor, &TPanel)>,
	player: Query<&Track<Skill<Active>>, With<Player>>,
	slot_map: Res<SlotMap<KeyCode>>,
	mouse_context: Res<State<MouseContext>>,
) {
	let active_slot_key = &player.get_single().map(|track| track.value.data.slot_key);
	let primed_slot_key = match mouse_context.get() {
		MouseContext::Primed(key) => slot_map.slots.get(key),
		_ => None,
	};

	for (entity, mut background_color, panel) in &mut buttons {
		let mut entity = commands.entity(entity);

		match (active_slot_key, primed_slot_key) {
			(Ok(active_key), _) if active_key == &panel.get(()) => {
				*background_color = TPanel::ACTIVE_COLOR.into();
				entity.insert(ColorOverride);
			}
			(_, Some(primed_key)) if primed_key == &panel.get(()) => {
				*background_color = TPanel::PANEL_COLORS.pressed.into();
				entity.insert(ColorOverride);
			}
			_ => {
				entity.remove::<ColorOverride>();
			}
		};
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::SlotKey,
		plugins::ingame_menu::{
			components::ColorOverride,
			traits::{
				colors::{HasPanelColors, PanelColors},
				get::Get,
			},
		},
		resources::SlotMap,
		states::MouseContext,
	};
	use bevy::{
		app::{App, Update},
		ecs::{bundle::Bundle, schedule::NextState},
		input::keyboard::KeyCode,
		prelude::default,
		render::color::Color,
	};

	#[derive(Component)]
	struct _Panel(pub SlotKey);

	impl HasActiveColor for _Panel {
		const ACTIVE_COLOR: Color = Color::BEIGE;
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
	) -> (App, Entity) {
		let mut app = App::new();

		app.add_systems(Update, panel_activity_colors_override::<_Panel>);
		app.add_state::<MouseContext>();
		app.insert_resource(SlotMap::<KeyCode>::new(slot_map));
		let player = app.world.spawn(Player::default()).id();
		let panel = app.world.spawn(bundle).id();

		if let Some(slot_key) = slot_key {
			let mut player = app.world.entity_mut(player);
			player.insert(Track::new(Skill::default().with(&Active {
				slot_key,
				..default()
			})));
		}

		(app, panel)
	}

	#[test]
	fn set_to_active_when_matching_skill_active() {
		let bundle = (BackgroundColor::from(Color::NONE), _Panel(SlotKey::Legs));
		let (mut app, panel) = setup(Some(SlotKey::Legs), bundle, []);

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
			_Panel(SlotKey::Legs),
			ColorOverride,
		);
		let (mut app, panel) = setup(Some(SlotKey::SkillSpawn), bundle, []);

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
			_Panel(SlotKey::Legs),
			ColorOverride,
		);
		let (mut app, panel) = setup(None, bundle, []);

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
		let bundle = (BackgroundColor::from(Color::NONE), _Panel(SlotKey::Legs));
		let (mut app, panel) = setup(None, bundle, [(KeyCode::Q, SlotKey::Legs, "")]);

		app.world
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Primed(KeyCode::Q));

		app.update();
		app.update();

		let panel = app.world.entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();

		assert_eq!(
			(_Panel::PANEL_COLORS.pressed, true),
			(color.0, panel.contains::<ColorOverride>())
		)
	}
}
