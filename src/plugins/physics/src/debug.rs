use crate::events::{InteractionEvent, Ray};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub(crate) struct Debug;

impl Debug {
	fn display_events(
		mut collision_events: EventReader<CollisionEvent>,
		mut contact_force_events: EventReader<ContactForceEvent>,
		mut ray_cast_events: EventReader<InteractionEvent<Ray>>,
	) {
		for collision_event in collision_events.read() {
			info!("Received collision event: {collision_event:?}");
		}

		for contact_force_event in contact_force_events.read() {
			info!("Received contact force event: {contact_force_event:?}");
		}

		for ray_cast_event in ray_cast_events.read() {
			info!("Received ray cast event: {ray_cast_event:?}");
		}
	}
}

impl Plugin for Debug {
	fn build(&self, app: &mut App) {
		app.add_plugins(RapierDebugRenderPlugin::default())
			.add_systems(Update, Self::display_events);
	}
}
