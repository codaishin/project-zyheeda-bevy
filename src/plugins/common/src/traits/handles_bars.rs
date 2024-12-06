use bevy::prelude::Bundle;

pub trait HandlesBars {
	type TBar: Bundle;

	fn new_bar() -> Self::TBar;
}
