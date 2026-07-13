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
		character_motion::ApplyMotion,
		collider::{ColliderRoot, ColliderShape},
		collision_domains::{Interactive, Physical},
		default_attributes::DefaultAttributes,
		effects::{Effects, force::ForceEffect},
		ground_target::GroundTarget,
		lifetime::{LifetimeTiedTo, TiedLifetimes},
		motion_controller::{MotionController, MotionControllerOf},
		set_velocity_forward::SetVelocityForward,
		skill::{Skill, SkillContactRoot, SkillProjectionRoot},
		target::Target,
		velocity::LinearVelocity,
		when_traveled::DestroyAfterDistanceTraveled,
	},
	messages::RayEvent,
	observers::{skill_prefab::SkillPrefab, update_blockers::UpdateBlockersObserver},
	resources::{root_collisions::RootCollisions, world_camera::WorldCamera},
	system_params::{
		config::ConfigParamMut,
		interactive::InteractiveParam,
		ray_caster::RayCasterMut,
		skill_agent::{SkillAgent, SkillAgentMut},
		update_root_collisions::UpdateRootCollisions,
	},
	systems::{
		apply_pull::ApplyPull,
		insert_affected::InsertAffected,
		interactions::push_ongoing_collisions::PushOngoingCollisions,
		interpolate_position::OverstepFraction,
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

		TSaveGame::register_savable_component::<ApplyMotion>(app);
		TSaveGame::register_savable_component::<Skill>(app);
		TSaveGame::register_savable_component::<Target>(app);
		TSaveGame::register_savable_component::<LinearVelocity>(app);
		TSaveGame::register_savable_component::<CharacterGravity>(app);

		app.configure_sets(
			Update,
			(
				PhysicsSystems::Prep,
				PhysicsSystems::Collisions,
				PhysicsSystems::Resolve,
				PhysicsSystems::Interpolate,
			)
				.chain(),
		);
		app.configure_sets(
			FixedUpdate,
			(
				PhysicsSystems::Prep,
				PhysicsSystems::Collisions,
				PhysicsSystems::Resolve,
				PhysicsSystems::Interpolate,
			)
				.chain(),
		);

		app
			// Rapier
			.add_plugins(RapierPhysicsPlugin::<NoUserData>::default().in_schedule(FixedPostUpdate))
			.register_required_components::<RigidBody, ColliderRoot>()
			.add_systems(
				Startup,
				set_rapier_time_step(Duration::from_secs(1) / self.target_fps),
			)
			.add_observer(LinearVelocity::apply)
			// World camera
			.init_resource::<WorldCamera>()
			.add_systems(
				Update,
				WorldCamera::reset_camera.in_set(PhysicsSystems::Prep),
			)
			// Character Motion
			.add_prefab_observer::<MotionControllerOf, ()>()
			.add_observer(MotionControllerOf::spawn)
			.add_systems(
				FixedUpdate,
				(
					FixedUpdate::delta.pipe(MotionController::set_translation),
					FixedUpdate::delta.pipe(MotionController::apply_gravity),
					FixedUpdate::delta.pipe(ApplyMotion::set_done),
				)
					.chain()
					.in_set(PhysicsSystems::Resolve),
			)
			.add_systems(
				Update,
				OverstepFraction::fixed
					.pipe(MotionControllerOf::interpolate_position)
					.in_set(PhysicsSystems::Interpolate),
			)
			// Animations
			.add_systems(
				Update,
				Target::update_pitch::<RayCasterMut, TAnimations::TAnimationsMut>
					.in_set(PhysicsSystems::Resolve),
			)
			// Skills
			.add_observer(Skill::prefab)
			// Colliders/Bodies
			.add_prefab_observer::<ColliderShape, ()>()
			.add_prefab_observer::<Body, ()>()
			.add_systems(
				Update,
				AsyncCollider::insert_collider
					.pipe(OnError::log)
					.in_set(PhysicsSystems::Prep),
			)
			.add_message::<RayEvent>()
			.init_resource::<RootCollisions<Physical>>()
			.init_resource::<RootCollisions<Interactive>>()
			// All effects
			.add_observer(Effects::insert)
			// Deal health damage
			.add_physics::<HealthDamageEffect, Life, TSaveGame>()
			.add_observer(HealthDamageEffect::update_blockers_observer)
			.add_systems(
				FixedUpdate,
				(Life::insert_from::<DefaultAttributes>, Life::despawn_dead)
					.chain()
					.in_set(PhysicsSystems::Resolve),
			)
			// Apply gravity effect
			.add_physics::<GravityEffect, GravityAffected, TSaveGame>()
			.add_observer(GravityEffect::update_blockers_observer)
			.add_systems(
				FixedUpdate,
				(
					GravityAffected::insert_from::<DefaultAttributes>,
					FixedUpdate::delta.pipe(GravityAffected::apply_pull),
				)
					.chain()
					.in_set(PhysicsSystems::Resolve),
			)
			// Apply force effect
			.add_physics::<ForceEffect, ForceAffected, TSaveGame>()
			.add_observer(ForceEffect::update_blockers_observer)
			.add_systems(
				FixedUpdate,
				ForceAffected::insert_from::<DefaultAttributes>.in_set(PhysicsSystems::Resolve),
			)
			// General Lifetime relationship
			.add_observer(LifetimeTiedTo::insert_on::<Anchor>)
			.add_observer(TiedLifetimes::despawn_relationships_on_remove)
			// Anchor
			.add_observer(AnchorDirty::process::<RayCasterMut>.pipe(OnError::log))
			.add_systems(Update, Anchor::mark_dirty.in_set(PhysicsSystems::Prep))
			.add_systems(
				Update,
				(
					// pre bake collider child relations
					ColliderRoot::link_children,
					// Skill spawning/lifetime
					(
						GroundTarget::set_position::<RayCasterMut>,
						DestroyAfterDistanceTraveled::system,
						SetVelocityForward::system,
					)
						.chain(),
				),
			)
			.add_systems(
				FixedUpdate,
				(
					// Collect physical collections
					(
						Blockable::apply_beam_blocks.pipe(OnError::log),
						RootCollisions::<Physical>::clear,
						FixedUpdate::delta
							.pipe(UpdateRootCollisions::<Physical>::prevent_tunneling)
							.pipe(OnError::log),
						UpdateRootCollisions::<Physical>::push_ongoing_collisions,
					)
						.chain(),
					// Collect interactive collisions
					(
						RootCollisions::<Interactive>::clear,
						UpdateRootCollisions::<Interactive>::push_ongoing_collisions,
					)
						.chain(),
				)
					.chain()
					.in_set(PhysicsSystems::Collisions),
			)
			.add_systems(
				FixedUpdate,
				apply_fragile_blocks.after(PhysicsSystems::Resolve),
			);
	}
}

fn set_rapier_time_step(time_per_frame: Duration) -> impl Fn(ResMut<TimestepMode>) {
	move |mut time_step_mode: ResMut<TimestepMode>| {
		*time_step_mode = TimestepMode::Fixed {
			dt: time_per_frame.as_secs_f32(),
			substeps: 1,
		}
	}
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone)]
pub enum PhysicsSystems {
	Prep,
	Collisions,
	Resolve,
	Interpolate,
}

impl<TDependencies> HandlesRaycast for PhysicsPlugin<TDependencies> {
	type TRaycastMut = RayCasterMut<'static, 'static>;
}

impl<TDependencies> HandlesPhysicsConfig for PhysicsPlugin<TDependencies> {
	type TConfigMut = ConfigParamMut<'static, 'static>;
}

impl<TDependencies> SystemSetDefinition for PhysicsPlugin<TDependencies> {
	type TSystemSet = PhysicsSystems;

	const SYSTEMS: PluginSystemSet<Self::TSystemSet> =
		PluginSystemSet::from_set(PhysicsSystems::Interpolate);
}

impl<TDependencies> HandlesMotion for PhysicsPlugin<TDependencies> {
	type TCharacterMotion = ApplyMotion;
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
	type TInteractions = InteractiveParam<'static>;
}
