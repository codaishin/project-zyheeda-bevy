use crate::components::{
	enemy::{Enemy, attacking::Attacking},
	player::Player,
};
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	components::ground_offset::GroundOffset,
	traits::{
		accessors::get::TryApplyOn,
		handles_physics::{Raycast, SolidObjects},
	},
	zyheeda_commands::ZyheedaCommands,
};

impl Enemy {
	pub(crate) fn attack_decision<TRaycast>(
		mut commands: ZyheedaCommands,
		mut raycast: StaticSystemParam<TRaycast>,
		players: Query<(Entity, &Transform), With<Player>>,
		enemies: Query<(Entity, &Self, &Transform, Option<&GroundOffset>)>,
	) where
		TRaycast: for<'w, 's> SystemParam<Item<'w, 's>: Raycast<SolidObjects>>,
	{
		let Ok((player, player_transform)) = players.single() else {
			return;
		};

		for (entity, enemy, transform, ground_offset) in &enemies {
			let mut attacking = || {
				let direction = player_transform.translation - transform.translation;
				if direction.length() > *enemy.attack_range {
					return None;
				};
				let direction = Dir3::try_from(direction).ok()?;
				let ground_offset = ground_offset.map(|GroundOffset(o)| *o).unwrap_or_default();
				let hit = raycast.raycast(SolidObjects {
					ray: Ray3d {
						origin: transform.translation + ground_offset,
						direction,
					},
					exclude: vec![entity],
					only_hoverable: false,
				});

				match hit {
					Some(hit) => Some(Attacking {
						has_los: hit.entity == player,
						player,
					}),
					None => Some(Attacking {
						has_los: false,
						player,
					}),
				}
			};

			commands.try_apply_on(&entity, move |mut e| match attacking() {
				Some(attacking) => {
					e.try_insert(attacking);
				}
				None => {
					e.try_remove::<Attacking>();
				}
			});
		}
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::components::{enemy::attacking::Attacking, player::Player};
	use common::{tools::Units, traits::handles_physics::RaycastHit};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	struct _Raycast {
		mock: Mock_Raycast,
	}

	#[automock]
	impl Raycast<SolidObjects> for _Raycast {
		fn raycast(&mut self, args: SolidObjects) -> Option<RaycastHit> {
			self.mock.raycast(args)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, Enemy::attack_decision::<ResMut<_Raycast>>);

		app
	}

	#[test]
	fn in_attack_range() {
		let mut app = setup();
		let player = app
			.world_mut()
			.spawn((Player, Transform::from_xyz(1., 2., 3.)))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				Enemy {
					attack_range: Units::from(5.),
					..default()
				},
				Transform::from_xyz(1., 2., 7.9),
			))
			.id();
		app.insert_resource(_Raycast::new().with_mock(|mock| {
			mock.expect_raycast()
				.once()
				.with(eq(SolidObjects {
					ray: Ray3d {
						origin: Vec3::new(1., 2., 7.9),
						direction: Dir3::try_from(Vec3::new(1., 2., 3.) - Vec3::new(1., 2., 7.9))
							.unwrap(),
					},
					exclude: vec![enemy],
					only_hoverable: false,
				}))
				.return_const(RaycastHit {
					entity: player,
					time_of_impact: 42.,
				});
		}));

		app.update();

		assert_eq!(
			Some(&Attacking {
				has_los: true,
				player
			}),
			app.world().entity(enemy).get::<Attacking>(),
		);
	}

	#[test]
	fn in_attack_range_from_ground_offset() {
		let mut app = setup();
		let player = app
			.world_mut()
			.spawn((Player, Transform::from_xyz(1., 2., 3.)))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				Enemy {
					attack_range: Units::from(5.),
					..default()
				},
				GroundOffset(Vec3::new(0., 2., 0.)),
				Transform::from_xyz(1., 2., 7.9),
			))
			.id();
		app.insert_resource(_Raycast::new().with_mock(|mock| {
			mock.expect_raycast()
				.once()
				.with(eq(SolidObjects {
					ray: Ray3d {
						origin: Vec3::new(1., 4., 7.9),
						direction: Dir3::try_from(Vec3::new(1., 2., 3.) - Vec3::new(1., 2., 7.9))
							.unwrap(),
					},
					exclude: vec![enemy],
					only_hoverable: false,
				}))
				.return_const(RaycastHit {
					entity: player,
					time_of_impact: 42.,
				});
		}));

		app.update();

		assert_eq!(
			Some(&Attacking {
				has_los: true,
				player
			}),
			app.world().entity(enemy).get::<Attacking>(),
		);
	}

	#[test]
	fn in_attack_range_without_los() {
		let mut app = setup();
		let player = app
			.world_mut()
			.spawn((Player, Transform::from_xyz(1., 2., 3.)))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				Enemy {
					attack_range: Units::from(5.),
					..default()
				},
				Transform::from_xyz(1., 2., 7.9),
			))
			.id();
		app.insert_resource(_Raycast::new().with_mock(|mock| {
			mock.expect_raycast().return_const(None);
		}));

		app.update();

		assert_eq!(
			Some(&Attacking {
				has_los: false,
				player
			}),
			app.world().entity(enemy).get::<Attacking>(),
		);
	}

	#[test]
	fn in_attack_range_without_los_due_to_other_entity_in_the_way() {
		let mut app = setup();
		let player = app
			.world_mut()
			.spawn((Player, Transform::from_xyz(1., 2., 3.)))
			.id();
		let other = app.world_mut().spawn_empty().id();
		let enemy = app
			.world_mut()
			.spawn((
				Enemy {
					attack_range: Units::from(5.),
					..default()
				},
				Transform::from_xyz(1., 2., 7.9),
			))
			.id();
		app.insert_resource(_Raycast::new().with_mock(|mock| {
			mock.expect_raycast().return_const(RaycastHit {
				entity: other,
				time_of_impact: 42.,
			});
		}));

		app.update();

		assert_eq!(
			Some(&Attacking {
				has_los: false,
				player
			}),
			app.world().entity(enemy).get::<Attacking>(),
		);
	}

	#[test]
	fn not_in_attack_range() {
		let mut app = setup();
		app.world_mut()
			.spawn((Player, Transform::from_xyz(1., 2., 3.)));
		let enemy = app
			.world_mut()
			.spawn((
				Enemy {
					attack_range: Units::from(5.),
					..default()
				},
				Transform::from_xyz(1., 2., 8.1),
			))
			.id();
		app.insert_resource(_Raycast::new().with_mock(|mock| {
			mock.expect_raycast().return_const(None);
		}));

		app.update();

		assert_eq!(None, app.world().entity(enemy).get::<Attacking>());
	}

	#[test]
	fn out_of_attack_range() {
		let mut app = setup();
		let player = app
			.world_mut()
			.spawn((Player, Transform::from_xyz(1., 2., 3.)))
			.id();
		let enemy = app
			.world_mut()
			.spawn((
				Enemy {
					attack_range: Units::from(5.),
					..default()
				},
				Transform::from_xyz(1., 2., 8.1),
				Attacking {
					has_los: false,
					player,
				},
			))
			.id();
		app.insert_resource(_Raycast::new().with_mock(|mock| {
			mock.expect_raycast().return_const(None);
		}));

		app.update();

		assert_eq!(None, app.world().entity(enemy).get::<Attacking>());
	}
}
