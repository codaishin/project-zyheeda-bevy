#[cfg(test)]
mod orbit_transform_on_mouse_motion_tests;

use bevy::{
	input::mouse::MouseMotion,
	prelude::{Component, EventReader, Query, Transform},
};

use crate::traits::orbit::Orbit;

pub fn orbit_transform_on_mouse_motion<TOrbitComponent: Orbit + Component>(
	mut mouse_motion: EventReader<MouseMotion>,
	mut query: Query<(&TOrbitComponent, &mut Transform)>,
) {
	for event in mouse_motion.iter() {
		for (orbit, mut transform) in query.iter_mut() {
			orbit.orbit(&mut transform, event.delta);
		}
	}
}
