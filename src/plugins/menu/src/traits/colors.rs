use bevy::color::Color;

pub(crate) struct PanelColors {
	pub(crate) disabled: ColorConfig,
	pub(crate) pressed: ColorConfig,
	pub(crate) hovered: ColorConfig,
	pub(crate) empty: ColorConfig,
	pub(crate) filled: ColorConfig,
}

impl PanelColors {
	pub(crate) const DEFAULT: PanelColors = PanelColors {
		disabled: ColorConfig {
			background: Color::srgba(0.1, 0.1, 0.1, 0.5),
			text: Color::srgba(0.2, 0.2, 0.2, 0.5),
		},
		pressed: ColorConfig {
			background: Color::srgb(0.35, 0.75, 0.35),
			text: Color::srgb(0.9, 0.9, 0.9),
		},
		hovered: ColorConfig {
			background: Color::srgb(0.25, 0.25, 0.25),
			text: Color::srgb(0.9, 0.9, 0.9),
		},
		filled: ColorConfig {
			background: Color::srgb(0.15, 0.15, 0.15),
			text: Color::srgb(0.9, 0.9, 0.9),
		},
		empty: ColorConfig {
			background: Color::srgb(0.35, 0.35, 0.35),
			text: Color::srgb(0.9, 0.9, 0.9),
		},
	};
}

impl Default for PanelColors {
	fn default() -> Self {
		Self::DEFAULT
	}
}

pub(crate) trait HasPanelColors {
	const PANEL_COLORS: PanelColors;
}

pub(crate) trait HasActiveColor {
	const ACTIVE_COLORS: ColorConfig;
}

pub(crate) trait HasQueuedColor {
	const QUEUED_COLORS: ColorConfig;
}

pub(crate) struct ColorConfig {
	pub(crate) background: Color,
	pub(crate) text: Color,
}

#[cfg(test)]
impl ColorConfig {
	pub(crate) const NO_COLORS: ColorConfig = ColorConfig {
		background: Color::NONE,
		text: Color::NONE,
	};
}
