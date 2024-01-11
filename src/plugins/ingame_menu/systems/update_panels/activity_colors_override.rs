use crate::{
	components::{Player, SlotKey, Track},
	plugins::ingame_menu::{
		components::ColorOverride,
		traits::{colors::HasActiveColor, get::Get},
	},
	skill::{Active, Skill},
};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		query::With,
		system::{Commands, Query},
	},
	ui::BackgroundColor,
};

pub fn panel_activity_colors_override<TPanel: HasActiveColor + Get<(), SlotKey> + Component>(
	mut commands: Commands,
	player: Query<&Track<Skill<Active>>, With<Player>>,
	mut buttons: Query<(Entity, &mut BackgroundColor, &TPanel)>,
) {
	let active_key = player.get_single().map(|track| track.value.data.slot_key);

	for (entity, mut color, panel) in &mut buttons {
		let mut entity = commands.entity(entity);
		match active_key {
			Ok(active_key) if active_key == panel.get(()) => {
				*color = TPanel::ACTIVE_COLOR.into();
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
		plugins::ingame_menu::{components::ColorOverride, traits::get::Get},
	};
	use bevy::{
		app::{App, Update},
		prelude::default,
		render::color::Color,
	};

	#[derive(Component)]
	struct _Panel(pub SlotKey);

	impl HasActiveColor for _Panel {
		const ACTIVE_COLOR: Color = Color::BEIGE;
	}

	impl Get<(), SlotKey> for _Panel {
		fn get(&self, _: ()) -> SlotKey {
			self.0
		}
	}

	#[test]
	fn set_to_active_when_matching_skill_active() {
		let mut app = App::new();

		app.world.spawn((
			Player::default(),
			Track::new(Skill::default().with(&Active {
				slot_key: SlotKey::Legs,
				..default()
			})),
		));
		let panel = app
			.world
			.spawn((BackgroundColor::from(Color::NONE), _Panel(SlotKey::Legs)))
			.id();

		app.add_systems(Update, panel_activity_colors_override::<_Panel>);
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
		let mut app = App::new();

		app.world.spawn((
			Player::default(),
			Track::new(Skill::default().with(&Active {
				slot_key: SlotKey::SkillSpawn,
				..default()
			})),
		));
		let panel = app
			.world
			.spawn((
				BackgroundColor::from(Color::NONE),
				_Panel(SlotKey::Legs),
				ColorOverride,
			))
			.id();

		app.add_systems(Update, panel_activity_colors_override::<_Panel>);
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
		let mut app = App::new();

		app.world.spawn(Player::default());
		let panel = app
			.world
			.spawn((
				BackgroundColor::from(Color::NONE),
				_Panel(SlotKey::Legs),
				ColorOverride,
			))
			.id();

		app.add_systems(Update, panel_activity_colors_override::<_Panel>);
		app.update();

		let panel = app.world.entity(panel);
		let color = panel.get::<BackgroundColor>().unwrap();

		assert_eq!(
			(Color::NONE, false),
			(color.0, panel.contains::<ColorOverride>())
		);
	}
}
