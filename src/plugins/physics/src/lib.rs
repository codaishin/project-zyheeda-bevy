mod app;
mod components;
mod messages;
mod observers;
mod physics_hooks;
mod resources;
mod system_params;
mod systems;
mod traits;

#[cfg(debug_assertions)]
mod debug;

use crate::{
	app::add_physics::AddPhysics,
	components::{
		affected::{force_affected::ForceAffected, gravity_affected::GravityAffected, life::Life},
		anchor::{Always, Anchor, Once},
		async_collider::AsyncConvexCollider,
		blockable::Blockable,
		character_motion::ApplyCharacterMotion,
		collider::ColliderShape,
		default_attributes::DefaultAttributes,
		effects::{Effects, force::ForceEffect},
		ground_target::GroundTarget,
		immobilized::Immobilized,
		interaction_target::{ColliderOfInteractionTarget, InteractionTarget},
		lifetime::{LifetimeTiedTo, TiedLifetimes},
		no_hover::NoMouseHover,
		physical_body::PhysicalBody,
		set_velocity_forward::SetVelocityForward,
		skill::{ContactInteractionTarget, ProjectionInteractionTarget, Skill},
		when_traveled::DestroyAfterDistanceTraveled,
		world_camera::WorldCamera,
	},
	messages::{BeamInteraction, RayEvent},
	observers::{skill_prefab::SkillPrefab, update_blockers::UpdateBlockersObserver},
	physics_hooks::check_hollow_colliders::CheckHollowColliders,
	resources::ongoing_interactions::OngoingInteractions,
	system_params::{
		skill_spawner::SkillSpawnerMut,
		update_ongoing_interactions::UpdateOngoingInteractions,
	},
	systems::{
		apply_pull::ApplyPull,
		insert_affected::InsertAffected,
		interactions::{
			push_beam_interactions::PushBeamInteractions,
			push_ongoing_collisions::PushOngoingCollisions,
		},
	},
	traits::ray_cast::RayCaster,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	systems::log::OnError,
	traits::{
		delta::Delta,
		handles_physics::{
			HandlesMotion,
			HandlesPhysicalAttributes,
			HandlesPhysicalEffectTargets,
			HandlesPhysicalObjects,
			HandlesRaycast,
			physical_bodies::HandlesPhysicalBodies,
		},
		handles_saving::HandlesSaving,
		handles_skill_physics::{
			HandlesNewPhysicalSkill,
			HandlesPhysicalSkillComponents,
			HandlesPhysicalSkillSpawnPoints,
		},
		prefab::AddPrefabObserver,
		thread_safe::ThreadSafe,
	},
};
use components::effects::{gravity::GravityEffect, health_damage::HealthDamageEffect};
use std::{marker::PhantomData, time::Duration};
use systems::interactions::apply_fragile_blocks::apply_fragile_blocks;
use traits::act_on::ActOn;

pub struct PhysicsPlugin<TDependencies> {
	target_fps: u32,
	_p: PhantomData<TDependencies>,
}

impl<TSaveGame> PhysicsPlugin<TSaveGame>
where
	TSaveGame: ThreadSafe + HandlesSaving,
{
	pub fn new(target_fps: u32, _: &TSaveGame) -> Self {
		Self {
			target_fps,
			_p: PhantomData,
		}
	}
}

impl<TSaveGame> Plugin for PhysicsPlugin<TSaveGame>
where
	TSaveGame: ThreadSafe + HandlesSaving,
{
	fn build(&self, app: &mut App) {
		#[cfg(debug_assertions)]
		app.add_plugins(crate::debug::Debug);

		TSaveGame::register_savable_component::<ApplyCharacterMotion>(app);
		TSaveGame::register_savable_component::<Skill>(app);

		app
			// Add/Configure rapier
			.add_plugins(RapierPhysicsPlugin::<CheckHollowColliders>::default())
			.add_systems(
				Startup,
				set_rapier_time_step(Duration::from_secs(1) / self.target_fps),
			)
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
			// Character Motion
			.add_systems(
				FixedUpdate,
				(
					FixedUpdate::delta.pipe(ApplyCharacterMotion::execute),
					FixedUpdate::delta.pipe(ApplyCharacterMotion::set_done),
				)
					.chain()
					.in_set(PhysicsSystems),
			)
			// Skills
			.register_required_components::<Skill, TSaveGame::TSaveEntityMarker>()
			.add_observer(Skill::prefab)
			// Colliders/Bodies
			.add_prefab_observer::<ColliderShape, ()>()
			.add_observer(ColliderOfInteractionTarget::link)
			.add_systems(
				PostUpdate,
				(
					PhysicalBody::prefab.after(TransformSystems::Propagate),
					AsyncConvexCollider::insert_collider.pipe(OnError::log),
				),
			)
			// All effects
			.add_observer(Effects::insert)
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
			// General Lifetime relationship
			.add_observer(LifetimeTiedTo::insert_on::<Anchor<Always>>)
			.add_observer(TiedLifetimes::despawn_relationships_on_remove)
			// Apply interactions
			.add_message::<RayEvent>()
			.add_message::<BeamInteraction>()
			.init_resource::<OngoingInteractions>()
			.add_systems(
				Update,
				(
					// Skill spawning/lifetime
					(
						GroundTarget::set_position,
						DestroyAfterDistanceTraveled::system,
						Anchor::<Once>::system.pipe(OnError::log),
						Anchor::<Always>::system.pipe(OnError::log),
						SetVelocityForward::system,
					)
						.chain(),
					// Physical effects
					(
						Blockable::beam_interactions.pipe(OnError::log),
						OngoingInteractions::clear,
						UpdateOngoingInteractions::push_beam_interactions,
						UpdateOngoingInteractions::push_ongoing_collisions,
					)
						.chain(),
				)
					.chain()
					.in_set(CollisionSystems),
			)
			.add_systems(
				Update,
				apply_fragile_blocks
					.after(PhysicsSystems)
					.after(CollisionSystems),
			);
	}
}

fn set_rapier_time_step(time_per_frame: Duration) -> impl Fn(ResMut<TimestepMode>) {
	move |mut time_step_mode: ResMut<TimestepMode>| {
		*time_step_mode = TimestepMode::Variable {
			max_dt: time_per_frame.as_secs_f32(),
			time_scale: 1.,
			substeps: 1,
		}
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

impl<TDependencies> HandlesPhysicalEffectTargets for PhysicsPlugin<TDependencies> {
	fn mark_as_effect_target<T>(app: &mut App)
	where
		T: Component,
	{
		app.register_required_components::<T, InteractionTarget>();
	}
}

impl<TDependencies> HandlesMotion for PhysicsPlugin<TDependencies> {
	type TCharacterMotion = ApplyCharacterMotion;
	type TCharacterImmobilized = Immobilized;
}

impl<TDependencies> HandlesPhysicalBodies for PhysicsPlugin<TDependencies> {
	type TBody = PhysicalBody;
}

impl<TDependencies> HandlesPhysicalSkillSpawnPoints for PhysicsPlugin<TDependencies> {
	type TSkillSpawnPointsMut<'w, 's> = SkillSpawnerMut<'w, 's>;
}

impl<TDependencies> HandlesNewPhysicalSkill for PhysicsPlugin<TDependencies> {
	type TSkillSpawnerMut<'w, 's> = SkillSpawnerMut<'w, 's>;
}

impl<TDependencies> HandlesPhysicalSkillComponents for PhysicsPlugin<TDependencies> {
	type TSkillContact = ContactInteractionTarget;
	type TSkillProjection = ProjectionInteractionTarget;
}
