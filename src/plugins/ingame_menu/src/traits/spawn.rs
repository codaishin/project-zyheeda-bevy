use bevy::ui::Style;

pub trait Spawn {
	fn spawn() -> (Style, Self);
}
