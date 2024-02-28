pub mod components;
mod events;
mod systems;
pub mod traits;

use bevy::{
	app::{App, Plugin, Update},
	ecs::schedule::{common_conditions::in_state, IntoSystemConfigs, States},
	time::Virtual,
};
use common::{components::Player, resources::CamRay};
use components::{Beam, CamOrbit, MovementConfig, Plasma, Projectile, SimpleMovement, VoidSphere};
use events::MoveInputEvent;
use prefabs::traits::RegisterPrefab;
use systems::{
	attack::{attack, execute_beam::execute_beam},
	chase::chase,
	enemy::enemy,
	face::face,
	follow::follow,
	move_on_orbit::move_on_orbit,
	movement::{
		execute_move::execute_move,
		move_player_on_event::move_player_on_event,
		trigger_event::trigger_move_input_event,
	},
	projectile::projectile_behavior,
	update_cool_downs::update_cool_downs,
	update_life_times::update_lifetimes,
};

pub struct BehaviorsPlugin<TCamActiveState: States + Clone + Send + Sync + 'static> {
	cam_behavior_state: TCamActiveState,
}

impl<TCamActiveState: States + Clone + Send + Sync + 'static> BehaviorsPlugin<TCamActiveState> {
	pub fn cam_behavior_if(cam_behavior_state: TCamActiveState) -> Self {
		Self { cam_behavior_state }
	}
}

impl<TCamActiveState: States + Clone + Send + Sync + 'static> Plugin
	for BehaviorsPlugin<TCamActiveState>
{
	fn build(&self, app: &mut App) {
		app.add_event::<MoveInputEvent>()
			.register_prefab::<Projectile<Plasma>>()
			.register_prefab::<VoidSphere>()
			.register_prefab::<Beam>()
			.add_systems(
				Update,
				(trigger_move_input_event::<CamRay>, move_player_on_event).chain(),
			)
			.add_systems(
				Update,
				(follow::<Player, CamOrbit>, move_on_orbit::<CamOrbit>)
					.run_if(in_state(self.cam_behavior_state.clone())),
			)
			.add_systems(
				Update,
				(update_cool_downs::<Virtual>, update_lifetimes::<Virtual>),
			)
			.add_systems(
				Update,
				(
					execute_move::<MovementConfig, SimpleMovement, Virtual>,
					face::<CamRay>,
				)
					.chain(),
			)
			.add_systems(Update, projectile_behavior::<Projectile<Plasma>>)
			.add_systems(Update, (enemy, chase::<MovementConfig>, attack).chain())
			.add_systems(Update, execute_beam);
	}
}
