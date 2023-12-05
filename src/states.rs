use bevy::ecs::schedule::States;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Default, States)]
pub enum GameState {
	#[default]
	None,
	Running,
	InGameMenu,
}
