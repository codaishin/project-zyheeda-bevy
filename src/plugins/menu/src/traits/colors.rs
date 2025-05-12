use bevy::color::Color;

pub struct PanelColors {
	pub pressed: Color,
	pub hovered: Color,
	pub empty: Color,
	pub filled: Color,
	pub text: Color,
}

impl PanelColors {
	pub const DEFAULT: PanelColors = PanelColors {
		pressed: Color::srgb(0.35, 0.75, 0.35),
		hovered: Color::srgb(0.25, 0.25, 0.25),
		filled: Color::srgb(0.15, 0.15, 0.15),
		empty: Color::srgb(0.35, 0.35, 0.35),
		text: Color::srgb(0.9, 0.9, 0.9),
	};
}

impl Default for PanelColors {
	fn default() -> Self {
		Self::DEFAULT
	}
}

pub trait HasPanelColors {
	const PANEL_COLORS: PanelColors;
}

pub trait HasActiveColor {
	const ACTIVE_COLOR: Color;
}

pub trait HasQueuedColor {
	const QUEUED_COLOR: Color;
}
