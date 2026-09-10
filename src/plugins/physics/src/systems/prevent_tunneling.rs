use crate::{
	components::{
		cast_rays::{CastRayFor, CastRays},
		collider::Colliders,
		collision_domains::Physical,
	},
	traits::send_collision_interaction::PushInteractingColliders,
};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};

impl<T> PreventTunneling for T where
	T: for<'w, 's> SystemParam<Item<'w, 's>: PushInteractingColliders>
{
}

pub(crate) trait PreventTunneling:
	for<'w, 's> SystemParam<Item<'w, 's>: PushInteractingColliders>
{
	fn prevent_tunneling(
		mut interactions: StaticSystemParam<Self>,
		colliders: Query<(&CastRays, &Colliders)>,
		physical: Query<&Physical>,
	) {
		for (cast_rays, colliders) in colliders {
			let Some(result) = cast_rays.results.get(&CastRayFor::TunnelingPrevention) else {
				continue;
			};

			let Some(hit) = result.hit else {
				continue;
			};

			for entity in colliders.iter() {
				let Ok(Physical::Contact) = physical.get(entity) else {
					continue;
				};

				interactions.push_interacting_colliders(hit, entity);
				interactions.push_interacting_colliders(entity, hit);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::components::{cast_rays::RayCastResult, collider::ColliderOf};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::collections::HashMap;
	use testing::{NestedMocks, SingleThreadedApp, fake_entity};

	#[derive(Resource, NestedMocks)]
	struct _OngoingCollisions {
		mock: Mock_OngoingCollisions,
	}

	impl Default for _OngoingCollisions {
		fn default() -> Self {
			Self::new().with_mock(|mock| {
				mock.expect_push_interacting_colliders().return_const(());
			})
		}
	}

	impl PushInteractingColliders for ResMut<'_, _OngoingCollisions> {
		fn push_interacting_colliders(&mut self, a: Entity, b: Entity) {
			self.mock.push_interacting_colliders(a, b);
		}
	}

	#[automock]
	impl PushInteractingColliders for _OngoingCollisions {
		fn push_interacting_colliders(&mut self, a: Entity, b: Entity) {
			self.mock.push_interacting_colliders(a, b);
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<_OngoingCollisions>();
		app.add_systems(Update, ResMut::<_OngoingCollisions>::prevent_tunneling);

		app
	}

	#[test]
	fn push_physical_contact_colliders() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((CastRays {
				results: HashMap::from([(
					CastRayFor::TunnelingPrevention,
					RayCastResult {
						hit: Some(fake_entity!(123)),
						..default()
					},
				)]),
			},))
			.id();
		let collider = app
			.world_mut()
			.spawn((ColliderOf(entity), Physical::Contact))
			.id();
		app.insert_resource(_OngoingCollisions::new().with_mock(|mock| {
			mock.expect_push_interacting_colliders()
				.times(1)
				.with(eq(collider), eq(fake_entity!(123)))
				.return_const(());
			mock.expect_push_interacting_colliders()
				.times(1)
				.with(eq(fake_entity!(123)), eq(collider))
				.return_const(());
		}));

		app.update();
	}
}
