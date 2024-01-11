use bevy::ecs::schedule::States;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, States)]
pub enum GameRunning {
	#[default]
	None,
	On,
	Off,
}

pub struct On;

pub struct Off;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, States)]
pub enum MouseContext {
	#[default]
	Default,
	UI,
}
