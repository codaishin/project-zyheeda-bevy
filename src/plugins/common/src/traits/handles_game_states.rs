use bevy::prelude::*;

pub trait HandlesGameStates {
	fn on_starting_new_game<TSystem, TMarker>(app: &mut App, systems: TSystem)
	where
		TSystem: IntoSystemConfigs<TMarker>;
}
