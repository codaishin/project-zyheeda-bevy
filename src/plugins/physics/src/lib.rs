pub mod components;
pub mod events;
pub mod traits;

mod observers;
mod resources;
mod systems;

use crate::{
	components::{
		blockable::Blockable,
		effect::force::ForceEffect,
		force_affected::ForceAffected,
		gravity_affected::GravityAffected,
		life::Life,
		motion::Motion,
	},
	observers::update_blockers::UpdateBlockersObserver,
	systems::interactions::act_on::ActOnSystem,
};
use bevy::{ecs::component::Mutable, prelude::*};
use bevy_rapier3d::prelude::Velocity;
use common::traits::{
	delta::Delta,
	handles_physics::{HandlesMotion, HandlesPhysicalObjects},
	handles_player::HandlesPlayer,
	handles_saving::{HandlesSaving, SavableComponent},
	register_derived_component::RegisterDerivedComponent,
	thread_safe::ThreadSafe,
};
use components::{
	active_beam::ActiveBeam,
	effect::{gravity::GravityEffect, health_damage::HealthDamageEffect},
	interacting_entities::InteractingEntities,
	running_interactions::RunningInteractions,
};
use events::{InteractionEvent, Ray};
use resources::{
	track_interaction_duplicates::TrackInteractionDuplicates,
	track_ray_interactions::TrackRayInteractions,
};
use std::marker::PhantomData;
use systems::{
	gravity_affected::apply_gravity_pull,
	interactions::{
		apply_fragile_blocks::apply_fragile_blocks,
		map_collision_events::map_collision_events_to,
		update_interacting_entities::update_interacting_entities,
	},
	ray_cast::{
		apply_interruptable_blocks::apply_interruptable_ray_blocks,
		execute_ray_caster::execute_ray_caster,
		map_ray_cast_results_to_interaction_event::map_ray_cast_result_to_interaction_events,
		send_interaction_events::send_interaction_events,
	},
};
use traits::act_on::ActOn;

pub struct PhysicsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TSaveGame, TPlayers> PhysicsPlugin<(TSaveGame, TPlayers)>
where
	TSaveGame: ThreadSafe + HandlesSaving,
	TPlayers: ThreadSafe + HandlesPlayer,
{
	pub fn from_plugin(_: &TSaveGame, _: &TPlayers) -> Self {
		Self(PhantomData)
	}
}

impl<TSaveGame, TPlayers> Plugin for PhysicsPlugin<(TSaveGame, TPlayers)>
where
	TSaveGame: ThreadSafe + HandlesSaving,
	TPlayers: ThreadSafe + HandlesPlayer,
{
	fn build(&self, app: &mut App) {
		TSaveGame::register_savable_component::<Motion>(app);

		app
			// Motion
			.register_derived_component::<Motion, Velocity>()
			.add_observer(Motion::zero_velocity_on_remove)
			.add_systems(
				FixedUpdate,
				FixedUpdate::delta
					.pipe(Motion::set_done)
					.in_set(PhysicsSystems),
			)
			// Deal health damage
			.register_required_components::<HealthDamageEffect, InteractingEntities>()
			.register_derived_component::<TPlayers::TPlayer, Life>()
			.register_derived_component::<TPlayers::TPlayer, GravityAffected>()
			.register_derived_component::<TPlayers::TPlayer, ForceAffected>()
			.add_observer(HealthDamageEffect::update_blockers)
			.add_physics::<HealthDamageEffect, Life, TSaveGame>()
			.add_systems(Update, Life::despawn_dead)
			// Apply gravity effect
			.register_required_components::<GravityEffect, InteractingEntities>()
			.add_observer(GravityEffect::update_blockers)
			.add_physics::<GravityEffect, GravityAffected, TSaveGame>()
			.add_systems(Update, Update::delta.pipe(apply_gravity_pull))
			// Apply force effect
			.register_required_components::<ForceEffect, InteractingEntities>()
			.add_observer(ForceEffect::update_blockers)
			.add_physics::<ForceEffect, ForceAffected, TSaveGame>()
			// Apply interactions
			.add_event::<InteractionEvent>()
			.add_event::<InteractionEvent<Ray>>()
			.init_resource::<TrackInteractionDuplicates>()
			.init_resource::<TrackRayInteractions>()
			.add_systems(
				Update,
				(
					apply_fragile_blocks,
					ActiveBeam::execute,
					execute_ray_caster
						.pipe(apply_interruptable_ray_blocks)
						.pipe(map_ray_cast_result_to_interaction_events)
						.pipe(send_interaction_events::<TrackRayInteractions>),
					map_collision_events_to::<InteractionEvent, TrackInteractionDuplicates>,
					update_interacting_entities, // must be last to ensure `InteractionEvent`s and `InteractingEntities` are synched
				)
					.chain()
					.in_set(CollisionSystems),
			);
	}
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct CollisionSystems;

trait AddPhysics {
	fn add_physics<TActor, TTarget, TSaveGame>(&mut self) -> &mut Self
	where
		TActor: ActOn<TTarget> + Component<Mutability = Mutable> + SavableComponent,
		TTarget: Component<Mutability = Mutable> + SavableComponent,
		TSaveGame: HandlesSaving;
}

impl AddPhysics for App {
	fn add_physics<TActor, TTarget, TSaveGame>(&mut self) -> &mut Self
	where
		TActor: ActOn<TTarget> + Component<Mutability = Mutable> + SavableComponent,
		TTarget: Component<Mutability = Mutable> + SavableComponent,
		TSaveGame: HandlesSaving,
	{
		TSaveGame::register_savable_component::<TActor>(self);
		TSaveGame::register_savable_component::<TTarget>(self);
		TSaveGame::register_savable_component::<RunningInteractions<TActor, TTarget>>(self);

		self.register_required_components::<TActor, RunningInteractions<TActor, TTarget>>()
			.add_systems(
				Update,
				(
					Update::delta.pipe(TActor::act_on::<TTarget>),
					RunningInteractions::<TActor, TTarget>::untrack_non_interacting_targets,
				)
					.chain()
					.in_set(PhysicsSystems)
					.after(CollisionSystems),
			)
	}
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone)]
pub struct PhysicsSystems;

impl<TDependencies> HandlesPhysicalObjects for PhysicsPlugin<TDependencies> {
	type TSystems = PhysicsSystems;
	type TPhysicalObjectComponent = Blockable;

	const SYSTEMS: Self::TSystems = PhysicsSystems;
}

impl<TDependencies> HandlesMotion for PhysicsPlugin<TDependencies> {
	type TMotion = Motion;
}
