use bevy::{ecs::system::ScheduleSystem, prelude::*};

pub trait HandlesGameStates {
	fn on_starting_new_game<TSystem, TMarker>(app: &mut App, systems: TSystem)
	where
		TSystem: IntoScheduleConfigs<ScheduleSystem, TMarker>;
}
