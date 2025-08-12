use crate::{
	components::{ColorOverride, dispatch_text_color::DispatchTextColor, ui_disabled::UIDisabled},
	tools::PanelState,
	traits::colors::HasPanelColors,
};
use bevy::prelude::*;
use common::{
	traits::accessors::get::{GetterRef, TryApplyOn},
	zyheeda_commands::ZyheedaCommands,
};

pub fn panel_colors<TPanel: Component + GetterRef<PanelState> + HasPanelColors>(
	mut commands: ZyheedaCommands,
	mut panels: Query<(Entity, &Interaction, &TPanel, Option<&UIDisabled>), Without<ColorOverride>>,
) {
	for (entity, interaction, panel, disabled) in &mut panels {
		let config = match (interaction, panel.get(), disabled) {
			(.., Some(UIDisabled)) => &TPanel::PANEL_COLORS.disabled,
			(Interaction::Pressed, ..) => &TPanel::PANEL_COLORS.pressed,
			(Interaction::Hovered, ..) => &TPanel::PANEL_COLORS.hovered,
			(_, PanelState::Empty, _) => &TPanel::PANEL_COLORS.empty,
			(_, PanelState::Filled, _) => &TPanel::PANEL_COLORS.filled,
		};

		commands.try_apply_on(&entity, |mut e| {
			e.try_insert((
				BackgroundColor::from(config.background),
				DispatchTextColor::from(config.text),
			));
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::dispatch_text_color::DispatchTextColor,
		traits::colors::{ColorConfig, PanelColors},
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
			disabled: ColorConfig {
				background: Color::srgb(0., 0., 1.),
				text: Color::srgb(0., 0.16, 1.),
			},
			pressed: ColorConfig {
				background: Color::srgb(1., 0., 0.),
				text: Color::srgb(1., 0.12, 0.),
			},
			hovered: ColorConfig {
				background: Color::srgb(0.5, 0., 0.),
				text: Color::srgb(0.5, 0.12, 0.),
			},
			empty: ColorConfig {
				background: Color::srgb(0.25, 0., 0.),
				text: Color::srgb(0.25, 0.11, 0.),
			},
			filled: ColorConfig {
				background: Color::srgb(0.125, 0., 0.),
				text: Color::srgb(0.125, 0.15, 0.),
			},
		};
	}

	impl HasPanelColors for _Filled {
		const PANEL_COLORS: PanelColors = PanelColors {
			disabled: ColorConfig {
				background: Color::srgb(0., 0., 1.),
				text: Color::NONE,
			},
			pressed: ColorConfig {
				background: Color::srgb(1., 0., 0.),
				text: Color::NONE,
			},
			hovered: ColorConfig {
				background: Color::srgb(0.5, 0., 0.),
				text: Color::NONE,
			},
			empty: ColorConfig {
				background: Color::srgb(0.25, 0., 0.),
				text: Color::NONE,
			},
			filled: ColorConfig {
				background: Color::srgb(0.125, 0., 0.),
				text: Color::NONE,
			},
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

		assert_eq!(
			(color, app.world().entity(agent).get::<DispatchTextColor>()),
			(
				_Empty::PANEL_COLORS.pressed.background,
				Some(&DispatchTextColor::from(_Empty::PANEL_COLORS.pressed.text))
			)
		);
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

		assert_eq!(
			(color, app.world().entity(agent).get::<DispatchTextColor>()),
			(
				_Empty::PANEL_COLORS.hovered.background,
				Some(&DispatchTextColor::from(_Empty::PANEL_COLORS.hovered.text))
			)
		);
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

		assert_eq!(
			(color, app.world().entity(agent).get::<DispatchTextColor>()),
			(
				_Empty::PANEL_COLORS.empty.background,
				Some(&DispatchTextColor::from(_Empty::PANEL_COLORS.empty.text))
			)
		);
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

		assert_eq!(
			(color, app.world().entity(agent).get::<DispatchTextColor>()),
			(
				_Filled::PANEL_COLORS.filled.background,
				Some(&DispatchTextColor::from(_Filled::PANEL_COLORS.filled.text))
			)
		);
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

	#[test]
	fn disabled() {
		let mut app = App::new();
		let agent = app
			.world_mut()
			.spawn((
				_Empty,
				Interaction::Pressed,
				BackgroundColor(Color::srgb(0.1, 0.2, 0.3)),
				UIDisabled,
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

		assert_eq!(
			(color, app.world().entity(agent).get::<DispatchTextColor>()),
			(
				_Empty::PANEL_COLORS.disabled.background,
				Some(&DispatchTextColor::from(_Empty::PANEL_COLORS.disabled.text))
			)
		);
	}
}
