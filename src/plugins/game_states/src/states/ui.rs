use bevy::prelude::*;

#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Default)]
pub(crate) enum Hud {
	#[default]
	Off,
	On,
}

#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Default)]
pub(crate) enum Inventory {
	#[default]
	Off,
	On,
}

#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Default)]
pub(crate) enum ComboOverview {
	#[default]
	Off,
	On,
}

#[derive(States, Debug, PartialEq, Eq, Hash, Clone, Default)]
pub(crate) enum Settings {
	#[default]
	Off,
	On,
}
