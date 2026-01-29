use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::messages::RayEvent;

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
}

impl Plugin for Debug {
	fn build(&self, app: &mut App) {
		app.add_plugins(RapierDebugRenderPlugin::default())
			.add_systems(Update, Self::display_events);
	}
}
