mod app;
mod components;
mod messages;
mod observers;
mod resources;
mod system_params;
mod systems;
mod traits;

#[cfg(test)]
mod tests;

#[cfg(debug_assertions)]
mod debug;

use crate::{
	app::add_physics::AddPhysics,
	components::{
		affected::{force_affected::ForceAffected, gravity_affected::GravityAffected, life::Life},
		anchor::{Anchor, AnchorDirty},
		async_collider::AsyncCollider,
		blockable::Blockable,
		body::Body,
		character_gravity::CharacterGravity,
		character_motion::ApplyCharacterMotion,
		collider::{ColliderRoot, ColliderShape},
		collision_domains::{Interactive, Physical},
		default_attributes::DefaultAttributes,
		effects::{Effects, force::ForceEffect},
		ground_target::GroundTarget,
		lifetime::{LifetimeTiedTo, TiedLifetimes},
		set_velocity_forward::SetVelocityForward,
		skill::{Skill, SkillContactRoot, SkillProjectionRoot},
		target::Target,
		velocity::LinearVelocity,
		when_traveled::DestroyAfterDistanceTraveled,
		world_camera::WorldCamera,
	},
	messages::RayEvent,
	observers::{skill_prefab::SkillPrefab, update_blockers::UpdateBlockersObserver},
	resources::ongoing_interactions::OngoingInteractions,
	system_params::{
		config::ConfigParamMut,
		interactive::InteractiveParam,
		ray_caster::RayCaster,
		skill_agent::{SkillAgent, SkillAgentMut},
		update_ongoing_interactions::UpdateOngoingInteractions,
	},
	systems::{
		apply_pull::ApplyPull,
		insert_affected::InsertAffected,
		interactions::push_ongoing_collisions::PushOngoingCollisions,
	},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::{
	systems::log::OnError,
	tools::plugin_system_set::PluginSystemSet,
	traits::{
		delta::Delta,
		handles_animations::HandlesAnimations,
		handles_physics::{
			HandlesInteractiveDetection,
			HandlesMotion,
			HandlesPhysicsConfig,
			HandlesRaycast,
		},
		handles_saving::HandlesSaving,
		handles_skill_physics::{
			HandlesNewPhysicalSkill,
			HandlesPhysicalSkillAgent,
			HandlesPhysicalSkillComponents,
		},
		prefab::AddPrefabObserver,
		system_set_definition::SystemSetDefinition,
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

impl<TSaveGame, TAnimations> PhysicsPlugin<(TSaveGame, TAnimations)>
where
	TSaveGame: ThreadSafe + HandlesSaving,
	TAnimations: ThreadSafe + HandlesAnimations,
{
	pub fn new(target_fps: u32, _: &TSaveGame, _: &TAnimations) -> Self {
		Self {
			target_fps,
			_p: PhantomData,
		}
	}
}

impl<TSaveGame, TAnimations> Plugin for PhysicsPlugin<(TSaveGame, TAnimations)>
where
	TSaveGame: ThreadSafe + HandlesSaving,
	TAnimations: ThreadSafe + HandlesAnimations,
{
	fn build(&self, app: &mut App) {
		#[cfg(debug_assertions)]
		app.add_plugins(crate::debug::Debug);

		TSaveGame::register_savable_component::<ApplyCharacterMotion>(app);
		TSaveGame::register_savable_component::<Skill>(app);
		TSaveGame::register_savable_component::<Target>(app);
		TSaveGame::register_savable_component::<LinearVelocity>(app);
		TSaveGame::register_savable_component::<CharacterGravity>(app);

		app
			// Rapier
			.add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
			.register_required_components::<RigidBody, ColliderRoot>()
			.add_systems(
				Startup,
				set_rapier_time_step(Duration::from_secs(1) / self.target_fps),
			)
			.add_observer(LinearVelocity::apply)
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
			.register_required_components::<KinematicCharacterController, CharacterGravity>()
			.add_systems(
				FixedUpdate,
				(
					FixedUpdate::delta.pipe(ApplyCharacterMotion::execute),
					FixedUpdate::delta.pipe(ApplyCharacterMotion::set_done),
					FixedUpdate::delta.pipe(CharacterGravity::apply),
				)
					.chain()
					.in_set(PhysicsSystems),
			)
			// Animations
			.add_systems(
				Update,
				Target::update_pitch::<RayCaster, TAnimations::TAnimationsMut>,
			)
			// Skills
			.register_required_components::<Skill, TSaveGame::TSaveEntityMarker>()
			.add_observer(Skill::prefab)
			// Colliders/Bodies
			.add_prefab_observer::<ColliderShape, ()>()
			.add_prefab_observer::<Body, ()>()
			.add_observer(ColliderRoot::link_children)
			.add_systems(Update, AsyncCollider::insert_collider.pipe(OnError::log))
			.add_message::<RayEvent>()
			.init_resource::<OngoingInteractions<Physical>>()
			.init_resource::<OngoingInteractions<Interactive>>()
			// All effects
			.add_observer(Effects::insert)
			// Deal health damage
			.add_physics::<HealthDamageEffect, Life, TSaveGame>()
			.add_observer(HealthDamageEffect::update_blockers_observer)
			.add_systems(
				Update,
				(Life::insert_from::<DefaultAttributes>, Life::despawn_dead)
					.chain()
					.in_set(PhysicsSystems),
			)
			// Apply gravity effect
			.add_physics::<GravityEffect, GravityAffected, TSaveGame>()
			.add_observer(GravityEffect::update_blockers_observer)
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
			.add_observer(ForceEffect::update_blockers_observer)
			.add_systems(
				Update,
				ForceAffected::insert_from::<DefaultAttributes>.in_set(PhysicsSystems),
			)
			// General Lifetime relationship
			.add_observer(LifetimeTiedTo::insert_on::<Anchor>)
			.add_observer(TiedLifetimes::despawn_relationships_on_remove)
			// Anchor
			.add_observer(AnchorDirty::process::<RayCaster>.pipe(OnError::log))
			.add_systems(Update, Anchor::mark_dirty.in_set(PhysicsSystems))
			.add_systems(
				Update,
				(
					// Skill spawning/lifetime
					(
						GroundTarget::set_position::<RayCaster>,
						DestroyAfterDistanceTraveled::system,
						SetVelocityForward::system,
					)
						.chain(),
					// Collect physical collections
					(
						Blockable::beam_interactions.pipe(OnError::log),
						OngoingInteractions::<Physical>::clear,
						Update::delta
							.pipe(UpdateOngoingInteractions::<Physical>::prevent_tunneling)
							.pipe(OnError::log),
						UpdateOngoingInteractions::<Physical>::push_ongoing_collisions,
					)
						.chain(),
					// Collect interactive collisions
					(
						OngoingInteractions::<Interactive>::clear,
						UpdateOngoingInteractions::<Interactive>::push_ongoing_collisions,
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
	type TRaycast = RayCaster<'static, 'static>;
}

impl<TDependencies> HandlesPhysicsConfig for PhysicsPlugin<TDependencies> {
	type TConfigMut = ConfigParamMut<'static, 'static>;
}

impl<TDependencies> SystemSetDefinition for PhysicsPlugin<TDependencies> {
	type TSystemSet = PhysicsSystems;

	const SYSTEMS: PluginSystemSet<Self::TSystemSet> = PluginSystemSet::from_set(PhysicsSystems);
}

impl<TDependencies> HandlesMotion for PhysicsPlugin<TDependencies> {
	type TCharacterMotion = ApplyCharacterMotion;
}

impl<TDependencies> HandlesPhysicalSkillAgent for PhysicsPlugin<TDependencies> {
	type TAgent = SkillAgent<'static, 'static>;
	type TAgentMut = SkillAgentMut<'static, 'static>;
}

impl<TDependencies> HandlesNewPhysicalSkill for PhysicsPlugin<TDependencies> {
	type TSkillSpawnerMut = SkillAgentMut<'static, 'static>;
}

impl<TDependencies> HandlesPhysicalSkillComponents for PhysicsPlugin<TDependencies> {
	type TSkillContact = SkillContactRoot;
	type TSkillProjection = SkillProjectionRoot;
}

impl<TDependencies> HandlesInteractiveDetection for PhysicsPlugin<TDependencies> {
	type TInteractive = InteractiveParam<'static>;
}
