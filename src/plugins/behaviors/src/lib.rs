pub mod components;
mod systems;
pub mod traits;

use bevy::{
	app::{Plugin, Update},
	ecs::schedule::{common_conditions::in_state, IntoSystemConfigs},
	time::Virtual,
};
use common::{
	components::{Plasma, Player, Projectile},
	states::GameRunning,
};
use components::{CamOrbit, MovementConfig, SimpleMovement};
use systems::{
	execute_move::execute_move,
	follow::follow,
	move_on_orbit::move_on_orbit,
	projectile::projectile_behavior,
	void_sphere::void_sphere_behavior,
};

pub struct BehaviorsPlugin;

impl Plugin for BehaviorsPlugin {
	fn build(&self, app: &mut bevy::prelude::App) {
		app.add_systems(
			Update,
			(follow::<Player, CamOrbit>, move_on_orbit::<CamOrbit>)
				.run_if(in_state(GameRunning::On)),
		)
		.add_systems(
			Update,
			(execute_move::<MovementConfig, SimpleMovement, Virtual>,),
		)
		.add_systems(Update, projectile_behavior::<Projectile<Plasma>>)
		.add_systems(Update, void_sphere_behavior::<MovementConfig>);
	}
}
