use bevy::ui::Style;

pub trait GetStyle {
	fn style(&self) -> Style;
}
