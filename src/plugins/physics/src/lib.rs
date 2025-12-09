pub mod events;

mod app;
mod components;
mod observers;
mod resources;
mod systems;
mod traits;

use crate::{
	app::add_physics::AddPhysics,
	components::{
		affected::{force_affected::ForceAffected, gravity_affected::GravityAffected, life::Life},
		blockable::Blockable,
		default_attributes::DefaultAttributes,
		effect::force::ForceEffect,
		fix_points::{Always, Anchor, Once, fix_point::FixPointSpawner},
		ground_target::GroundTarget,
		motion::Motion,
		no_hover::NoMouseHover,
		set_motion_forward::SetMotionForward,
		skill_prefabs::{skill_contact::SkillContact, skill_projection::SkillProjection},
		when_traveled::DestroyAfterDistanceTraveled,
		world_camera::WorldCamera,
	},
	observers::update_blockers::UpdateBlockersObserver,
	systems::{apply_pull::ApplyPull, insert_affected::InsertAffected},
	traits::ray_cast::RayCaster,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::Velocity;
use common::{
	components::{child_of_persistent::ChildOfPersistent, persistent_entity::PersistentEntity},
	systems::log::OnError,
	traits::{
		delta::Delta,
		handles_physics::{
			HandlesMotion,
			HandlesPhysicalAttributes,
			HandlesPhysicalObjects,
			HandlesRaycast,
		},
		handles_saving::HandlesSaving,
		handles_skill_behaviors::{
			Contact,
			HandlesSkillBehaviors,
			Projection,
			SkillEntities,
			SkillRoot,
		},
		prefab::AddPrefabObserver,
		register_derived_component::RegisterDerivedComponent,
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};
use components::{
	active_beam::ActiveBeam,
	effect::{gravity::GravityEffect, health_damage::HealthDamageEffect},
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

impl<TSaveGame> PhysicsPlugin<TSaveGame>
where
	TSaveGame: ThreadSafe + HandlesSaving,
{
	pub fn from_plugin(_: &TSaveGame) -> Self {
		Self(PhantomData)
	}
}

impl<TSaveGame> Plugin for PhysicsPlugin<TSaveGame>
where
	TSaveGame: ThreadSafe + HandlesSaving,
{
	fn build(&self, app: &mut App) {
		TSaveGame::register_savable_component::<Motion>(app);
		TSaveGame::register_savable_component::<SkillContact>(app);
		TSaveGame::register_savable_component::<SkillProjection>(app);

		app
			// World camera
			.add_observer(WorldCamera::remove_old_cameras)
			.add_systems(
				Update,
				(
					WorldCamera::reset_camera,
					WorldCamera::update_ray.pipe(OnError::log),
				)
					.chain()
					.in_set(PhysicsSystems),
			)
			// Motion
			.register_derived_component::<Motion, Velocity>()
			.add_observer(Motion::zero_velocity_on_remove)
			.add_systems(
				FixedUpdate,
				FixedUpdate::delta
					.pipe(Motion::set_done)
					.in_set(PhysicsSystems),
			)
			// Skills
			.register_required_components::<SkillContact, TSaveGame::TSaveEntityMarker>()
			.register_required_components::<SkillProjection, TSaveGame::TSaveEntityMarker>()
			.add_prefab_observer::<SkillContact, ()>()
			.add_prefab_observer::<SkillProjection, ()>()
			// Deal health damage
			.add_physics::<HealthDamageEffect, Life, TSaveGame>()
			.add_observer(HealthDamageEffect::update_blockers)
			.add_systems(
				Update,
				(Life::insert_from::<DefaultAttributes>, Life::despawn_dead)
					.chain()
					.in_set(PhysicsSystems),
			)
			// Apply gravity effect
			.add_physics::<GravityEffect, GravityAffected, TSaveGame>()
			.add_observer(GravityEffect::update_blockers)
			.add_systems(
				Update,
				(
					GravityAffected::insert_from::<DefaultAttributes>,
					Update::delta.pipe(GravityAffected::apply_pull),
				)
					.chain()
					.in_set(PhysicsSystems),
			)
			// Apply force effect
			.add_physics::<ForceEffect, ForceAffected, TSaveGame>()
			.add_observer(ForceEffect::update_blockers)
			.add_systems(
				Update,
				ForceAffected::insert_from::<DefaultAttributes>.in_set(PhysicsSystems),
			)
			// Apply interactions
			.add_event::<InteractionEvent>()
			.add_event::<InteractionEvent<Ray>>()
			.init_resource::<TrackInteractionDuplicates>()
			.init_resource::<TrackRayInteractions>()
			.add_systems(
				Update,
				(
					// Skill spawning/lifetime
					(
						FixPointSpawner::insert_fix_points,
						GroundTarget::set_position,
						DestroyAfterDistanceTraveled::system,
						SkillContact::update_range,
						Anchor::<Once>::system.pipe(OnError::log),
						Anchor::<Always>::system.pipe(OnError::log),
						SetMotionForward::system,
					)
						.chain(),
					// Physical effects
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
						.chain(),
				)
					.chain()
					.in_set(CollisionSystems),
			);
	}
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct CollisionSystems;

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone)]
pub struct PhysicsSystems;

impl<TDependencies> HandlesRaycast for PhysicsPlugin<TDependencies> {
	type TWorldCamera = WorldCamera;
	type TNoMouseHover = NoMouseHover;
	type TRaycast<'world, 'state> = RayCaster<'world, 'state>;
}

impl<TDependencies> HandlesPhysicalAttributes for PhysicsPlugin<TDependencies> {
	type TDefaultAttributes = DefaultAttributes;
}

impl<TDependencies> HandlesPhysicalObjects for PhysicsPlugin<TDependencies> {
	type TSystems = PhysicsSystems;
	type TPhysicalObjectComponent = Blockable;

	const SYSTEMS: Self::TSystems = PhysicsSystems;
}

impl<TDependencies> HandlesMotion for PhysicsPlugin<TDependencies> {
	type TMotion = Motion;
}

impl<TDependencies> HandlesSkillBehaviors for PhysicsPlugin<TDependencies> {
	type TSkillContact = SkillContact;
	type TSkillProjection = SkillProjection;

	fn spawn_skill(
		commands: &mut ZyheedaCommands,
		contact: Contact,
		projection: Projection,
	) -> SkillEntities {
		let persistent_contact = PersistentEntity::default();
		let contact = commands
			.spawn((SkillContact::from(contact), persistent_contact))
			.id();
		let projection = commands
			.spawn((
				SkillProjection::from(projection),
				ChildOfPersistent(persistent_contact),
			))
			.id();

		SkillEntities {
			root: SkillRoot {
				persistent_entity: persistent_contact,
				entity: contact,
			},
			contact,
			projection,
		}
	}
}
