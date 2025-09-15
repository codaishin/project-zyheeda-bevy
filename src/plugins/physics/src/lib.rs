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
	systems::{apply_pull::ApplyPull, interactions::act_on::ActOnSystem},
};
use bevy::{ecs::component::Mutable, prelude::*};
use bevy_rapier3d::prelude::Velocity;
use common::traits::{
	delta::Delta,
	handles_enemies::HandlesEnemies,
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

impl<TSaveGame, TAgents> PhysicsPlugin<(TSaveGame, TAgents)>
where
	TSaveGame: ThreadSafe + HandlesSaving,
	TAgents: ThreadSafe + HandlesPlayer + HandlesEnemies,
{
	pub fn from_plugin(_: &TSaveGame, _: &TAgents) -> Self {
		Self(PhantomData)
	}
}

impl<TSaveGame, TAgents> Plugin for PhysicsPlugin<(TSaveGame, TAgents)>
where
	TSaveGame: ThreadSafe + HandlesSaving,
	TAgents: ThreadSafe + HandlesPlayer + HandlesEnemies,
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
			.register_derived_component::<TAgents::TPlayer, Life>()
			.register_derived_component::<TAgents::TEnemy, Life>()
			.add_physics::<HealthDamageEffect, Life, TSaveGame>()
			.add_observer(HealthDamageEffect::update_blockers)
			.add_systems(Update, Life::despawn_dead.in_set(PhysicsSystems))
			// Apply gravity effect
			.register_derived_component::<TAgents::TPlayer, GravityAffected>()
			.register_derived_component::<TAgents::TEnemy, GravityAffected>()
			.add_physics::<GravityEffect, GravityAffected, TSaveGame>()
			.add_observer(GravityEffect::update_blockers)
			.add_systems(
				Update,
				Update::delta
					.pipe(GravityAffected::apply_pull)
					.in_set(PhysicsSystems),
			)
			// Apply force effect
			.register_derived_component::<TAgents::TPlayer, ForceAffected>()
			.register_derived_component::<TAgents::TEnemy, ForceAffected>()
			.add_physics::<ForceEffect, ForceAffected, TSaveGame>()
			.add_observer(ForceEffect::update_blockers)
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

		self.register_required_components::<TActor, InteractingEntities>();
		self.register_required_components::<TActor, RunningInteractions<TActor, TTarget>>();
		self.add_systems(
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
