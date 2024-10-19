use crate::{components::ColorOverride, tools::PanelState, traits::colors::HasPanelColors};
use bevy::{
	ecs::{component::Component, query::Without, system::Query},
	ui::{BackgroundColor, Interaction},
};
use common::traits::accessors::get::GetterRef;

pub fn panel_colors<TPanel: Component + GetterRef<PanelState> + HasPanelColors>(
	mut panels: Query<(&mut BackgroundColor, &Interaction, &TPanel), Without<ColorOverride>>,
) {
	let colors = TPanel::PANEL_COLORS;
	for (mut color, interaction, panel) in &mut panels {
		*color = match (interaction, panel.get()) {
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
	use crate::traits::colors::PanelColors;
	use bevy::{
		app::{App, Update},
		color::Color,
		ui::{BackgroundColor, Interaction},
	};

	#[derive(Component)]
	struct _Empty;

	impl GetterRef<PanelState> for _Empty {
		fn get(&self) -> &PanelState {
			&PanelState::Empty
		}
	}

	#[derive(Component)]
	struct _Filled;

	impl GetterRef<PanelState> for _Filled {
		fn get(&self) -> &PanelState {
			&PanelState::Filled
		}
	}

	impl HasPanelColors for _Empty {
		const PANEL_COLORS: PanelColors = PanelColors {
			pressed: Color::srgb(1., 0., 0.),
			hovered: Color::srgb(0.5, 0., 0.),
			empty: Color::srgb(0.25, 0., 0.),
			filled: Color::srgb(0.125, 0., 0.),
			text: Color::srgb(0.0625, 0., 0.),
		};
	}

	impl HasPanelColors for _Filled {
		const PANEL_COLORS: PanelColors = PanelColors {
			pressed: Color::srgb(1., 0., 0.),
			hovered: Color::srgb(0.5, 0., 0.),
			empty: Color::srgb(0.25, 0., 0.),
			filled: Color::srgb(0.125, 0., 0.),
			text: Color::srgb(0.0625, 0., 0.),
		};
	}

	#[test]
	fn pressed() {
		let mut app = App::new();
		let agent = app
			.world_mut()
			.spawn((
				_Empty,
				Interaction::Pressed,
				BackgroundColor(Color::srgb(0.1, 0.2, 0.3)),
			))
			.id();

		app.add_systems(Update, panel_colors::<_Empty>);
		app.update();

		let color = app
			.world()
			.entity(agent)
			.get::<BackgroundColor>()
			.unwrap()
			.0;

		assert_eq!(color, _Empty::PANEL_COLORS.pressed);
	}

	#[test]
	fn hovered() {
		let mut app = App::new();
		let agent = app
			.world_mut()
			.spawn((
				_Empty,
				Interaction::Hovered,
				BackgroundColor(Color::srgb(0.1, 0.2, 0.3)),
			))
			.id();

		app.add_systems(Update, panel_colors::<_Empty>);
		app.update();

		let color = app
			.world()
			.entity(agent)
			.get::<BackgroundColor>()
			.unwrap()
			.0;

		assert_eq!(color, _Empty::PANEL_COLORS.hovered);
	}

	#[test]
	fn no_interaction_and_empty() {
		let mut app = App::new();
		let agent = app
			.world_mut()
			.spawn((
				_Empty,
				Interaction::None,
				BackgroundColor(Color::srgb(0.1, 0.2, 0.3)),
			))
			.id();

		app.add_systems(Update, panel_colors::<_Empty>);
		app.update();

		let color = app
			.world()
			.entity(agent)
			.get::<BackgroundColor>()
			.unwrap()
			.0;

		assert_eq!(color, _Empty::PANEL_COLORS.empty);
	}

	#[test]
	fn no_interaction_and_not_empty() {
		let mut app = App::new();
		let agent = app
			.world_mut()
			.spawn((
				_Filled,
				Interaction::None,
				BackgroundColor(Color::srgb(0.1, 0.2, 0.3)),
			))
			.id();

		app.add_systems(Update, panel_colors::<_Filled>);
		app.update();

		let color = app
			.world()
			.entity(agent)
			.get::<BackgroundColor>()
			.unwrap()
			.0;

		assert_eq!(color, _Empty::PANEL_COLORS.filled);
	}

	#[test]
	fn ignore_when_color_override_set() {
		let mut app = App::new();
		let agent = app
			.world_mut()
			.spawn((
				_Empty,
				Interaction::Pressed,
				BackgroundColor(Color::srgb(0.1, 0.2, 0.3)),
				ColorOverride,
			))
			.id();

		app.add_systems(Update, panel_colors::<_Empty>);
		app.update();

		let color = app
			.world()
			.entity(agent)
			.get::<BackgroundColor>()
			.unwrap()
			.0;

		assert_eq!(color, Color::srgb(0.1, 0.2, 0.3));
	}
}
