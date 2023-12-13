use crate::plugins::ingame_menu::{
	tools::PanelState,
	traits::{colors::GetPanelColors, panel_state::GetPanelState},
};
use bevy::{
	ecs::{component::Component, system::Query},
	ui::{BackgroundColor, Interaction},
};

pub fn panel_color<TPanel: Component + GetPanelState, TGetPAnelColors: GetPanelColors>(
	mut panels: Query<(&mut BackgroundColor, &Interaction, &TPanel)>,
) {
	let colors = TGetPAnelColors::get_panel_colors();
	for (mut color, interaction, panel) in &mut panels {
		*color = match (interaction, panel.get_panel_state()) {
			(Interaction::Pressed, ..) => colors.pressed.into(),
			(Interaction::Hovered, ..) => colors.hovered.into(),
			(.., PanelState::Empty) => colors.empty.into(),
			(.., PanelState::Filled) => colors.filled.into(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::plugins::ingame_menu::traits::colors::PanelColors;
	use bevy::{
		app::{App, Update},
		render::color::Color,
		ui::{BackgroundColor, Interaction},
	};

	#[derive(Component)]
	struct _Empty;

	impl GetPanelState for _Empty {
		fn get_panel_state(&self) -> PanelState {
			PanelState::Empty
		}
	}

	#[derive(Component)]
	struct _Filled;

	impl GetPanelState for _Filled {
		fn get_panel_state(&self) -> PanelState {
			PanelState::Filled
		}
	}

	struct _Colors;

	impl GetPanelColors for _Colors {
		fn get_panel_colors() -> PanelColors {
			PanelColors {
				pressed: Color::rgb(1., 0., 0.),
				hovered: Color::rgb(0.5, 0., 0.),
				empty: Color::rgb(0.25, 0., 0.),
				filled: Color::rgb(0.125, 0., 0.),
			}
		}
	}

	#[test]
	fn pressed() {
		let mut app = App::new();
		let agent = app
			.world
			.spawn((
				_Empty,
				Interaction::Pressed,
				BackgroundColor(Color::rgb(0.1, 0.2, 0.3)),
			))
			.id();

		app.add_systems(Update, panel_color::<_Empty, _Colors>);
		app.update();

		let color = app.world.entity(agent).get::<BackgroundColor>().unwrap().0;

		assert_eq!(color, _Colors::get_panel_colors().pressed);
	}

	#[test]
	fn hovered() {
		let mut app = App::new();
		let agent = app
			.world
			.spawn((
				_Empty,
				Interaction::Hovered,
				BackgroundColor(Color::rgb(0.1, 0.2, 0.3)),
			))
			.id();

		app.add_systems(Update, panel_color::<_Empty, _Colors>);
		app.update();

		let color = app.world.entity(agent).get::<BackgroundColor>().unwrap().0;

		assert_eq!(color, _Colors::get_panel_colors().hovered);
	}

	#[test]
	fn no_interaction_and_empty() {
		let mut app = App::new();
		let agent = app
			.world
			.spawn((
				_Empty,
				Interaction::None,
				BackgroundColor(Color::rgb(0.1, 0.2, 0.3)),
			))
			.id();

		app.add_systems(Update, panel_color::<_Empty, _Colors>);
		app.update();

		let color = app.world.entity(agent).get::<BackgroundColor>().unwrap().0;

		assert_eq!(color, _Colors::get_panel_colors().empty);
	}

	#[test]
	fn no_interaction_and_not_empty() {
		let mut app = App::new();
		let agent = app
			.world
			.spawn((
				_Filled,
				Interaction::None,
				BackgroundColor(Color::rgb(0.1, 0.2, 0.3)),
			))
			.id();

		app.add_systems(Update, panel_color::<_Filled, _Colors>);
		app.update();

		let color = app.world.entity(agent).get::<BackgroundColor>().unwrap().0;

		assert_eq!(color, _Colors::get_panel_colors().filled);
	}
}
