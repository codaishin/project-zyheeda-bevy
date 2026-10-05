use crate::{components::self_skill_scale::SelfSkillScale, messages::RayEvent};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::prelude::*;

pub(crate) struct Debug;

impl Debug {
	fn display_events(
		mut collision_messages: MessageReader<CollisionEvent>,
		mut contact_force_messages: MessageReader<ContactForceEvent>,
		mut ray_cast_messages: MessageReader<RayEvent>,
	) {
		for collision_message in collision_messages.read() {
			info!("Received collision message: {collision_message:?}");
		}

		for contact_force_message in contact_force_messages.read() {
			info!("Received contact force message: {contact_force_message:?}");
		}

		for ray_cast_message in ray_cast_messages.read() {
			info!("Received ray cast message: {ray_cast_message:?}");
		}
	}

	fn control_collider_debug(
		mut commands: ZyheedaCommands,
		colliders: Query<(Entity, &GlobalTransform, Option<&ColliderDebug>), With<Collider>>,
		agents: Query<&GlobalTransform, With<SelfSkillScale>>,
	) {
		for (entity, collider_transform, current_debug) in colliders {
			let collider_translation = collider_transform.translation();
			let debug = agents.iter().any(|agent_transform| {
				(agent_transform.translation() - collider_translation).length() < 10.
			});
			let debug = match debug {
				true => ColliderDebug::AlwaysRender,
				false => ColliderDebug::NeverRender,
			};

			if current_debug == Some(&debug) {
				continue;
			};

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(debug);
			});
		}
	}
}

impl Plugin for Debug {
	fn build(&self, app: &mut App) {
		app.add_plugins(RapierDebugRenderPlugin::default())
			.add_systems(Update, (Self::display_events, Self::control_collider_debug));
	}
}
