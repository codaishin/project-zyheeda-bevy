use crate::components::{
	attack_movement::AttackMovement,
	enemy::Enemy,
	movement_config::MovementConfig,
	player::Player,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::{
	tools::Units,
	traits::{
		accessors::get::EntityContextMut,
		handles_movement::{
			CurrentMovement,
			Movement,
			MovementTarget,
			StartMovement,
			StopMovement,
		},
	},
};

impl Enemy {
	pub(crate) fn chase_player<TMovement>(
		mut movement: StaticSystemParam<TMovement>,
		player: Query<&Transform, With<Player>>,
		enemies: Query<(Entity, &Self, &MovementConfig, &Transform), Without<AttackMovement>>,
	) where
		TMovement: for<'c> EntityContextMut<
				Movement,
				TContext<'c>: StartMovement + StopMovement + CurrentMovement,
			>,
	{
		let Ok(player) = player.single() else {
			return;
		};

		for (entity, enemy, config, transform) in &enemies {
			let ctx = TMovement::get_entity_context_mut(&mut movement, entity, Movement);
			let Some(mut ctx) = ctx else {
				continue;
			};
			let current_movement = ctx.current_movement();
			let in_chase_range = enemy.in_chase_range(transform, player);

			match (in_chase_range, current_movement) {
				(false, Some(_)) => {
					ctx.stop();
				}
				(true, None) => {
					ctx.start(
						player.translation,
						config.collider_radius,
						config.speed,
						config.animation.clone(),
					);
				}
				(true, Some(MovementTarget::Point(point))) if point != player.translation => {
					ctx.start(
						player.translation,
						config.collider_radius,
						config.speed,
						config.animation.clone(),
					);
				}
				_ => {}
			}
		}
	}

	fn in_chase_range(&self, enemy_transform: &Transform, player_transform: &Transform) -> bool {
		let distance = (enemy_transform.translation - player_transform.translation).length();
		distance < *self.aggro_range && distance > *self.min_target_distance.unwrap_or(Units::ZERO)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{
		attack_movement::AttackMovement,
		movement_config::MovementConfig,
		player::Player,
	};
	use common::{
		tools::{Units, UnitsPerSecond},
		traits::{
			animation::{Animation, AnimationAsset, PlayMode},
			handles_movement::MovementTarget,
		},
	};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};
	use test_case::test_case;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component, NestedMocks)]
	struct _Movement {
		mock: Mock_Movement,
	}

	impl StartMovement for _Movement {
		fn start<T>(
			&mut self,
			target: T,
			radius: Units,
			speed: UnitsPerSecond,
			animation: Option<Animation>,
		) where
			T: Into<MovementTarget> + 'static,
		{
			self.mock.start(target, radius, speed, animation);
		}
	}

	impl StopMovement for _Movement {
		fn stop(&mut self) {
			self.mock.stop();
		}
	}

	impl CurrentMovement for _Movement {
		fn current_movement(&self) -> Option<MovementTarget> {
			self.mock.current_movement()
		}
	}

	mock! {
		_Movement {}
		impl StartMovement for _Movement {
			fn start<T>(
				&mut self,
				target: T,
				radius:Units,
				speed: UnitsPerSecond,
				animation: Option<Animation>,
			) where T: Into<MovementTarget> + 'static;
		}
		impl StopMovement for _Movement {
			fn stop(&mut self);
		}
		impl CurrentMovement for _Movement {
			fn current_movement(&self) -> Option<MovementTarget>;
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, Enemy::chase_player::<Query<&mut _Movement>>);

		app
	}

	#[test_case(Some(MovementTarget::Point(Vec3::new(4., 5., 6.))); "and current movement is different")]
	#[test_case(None; "and no current movement")]
	fn chase_player_when_in_aggro_range(current_movement: Option<MovementTarget>) {
		let speed = UnitsPerSecond::from(42.);
		let collider_radius = Units::from(11.);
		let animation = Some(Animation::new(
			AnimationAsset::from("my/asset"),
			PlayMode::Replay,
		));
		let mut app = setup();
		app.world_mut()
			.spawn((Transform::from_xyz(1., 2., 3.), Player));
		app.world_mut().spawn((
			Transform::from_xyz(1., 2., 6.9),
			Enemy {
				aggro_range: Units::from(4.),
				attack_range: Units::from(3.),
				min_target_distance: None,
			},
			MovementConfig {
				speed,
				collider_radius,
				animation: animation.clone(),
			},
			_Movement::new().with_mock(move |mock| {
				mock.expect_start()
					.once()
					.with(
						eq(Vec3::new(1., 2., 3.)),
						eq(collider_radius),
						eq(speed),
						eq(animation.clone()),
					)
					.return_const(());
				mock.expect_current_movement()
					.return_const(current_movement);
			}),
		));

		app.update();
	}

	#[test]
	fn do_not_chase_player_when_out_of_aggro_range() {
		let speed = UnitsPerSecond::from(42.);
		let collider_radius = Units::from(11.);
		let animation = Some(Animation::new(
			AnimationAsset::from("my/asset"),
			PlayMode::Replay,
		));
		let mut app = setup();
		app.world_mut()
			.spawn((Transform::from_xyz(1., 2., 3.), Player));
		app.world_mut().spawn((
			Transform::from_xyz(1., 2., 7.1),
			Enemy {
				aggro_range: Units::from(4.),
				attack_range: Units::from(3.),
				min_target_distance: None,
			},
			MovementConfig {
				speed,
				collider_radius,
				animation: animation.clone(),
			},
			_Movement::new().with_mock(move |mock| {
				mock.expect_start::<Vec3>().never();
				mock.expect_start::<Dir3>().never();
				mock.expect_stop().never();
				mock.expect_current_movement().return_const(None);
			}),
		));

		app.update();
	}

	#[test]
	fn stop_chase_player_when_out_of_aggro_range() {
		let speed = UnitsPerSecond::from(42.);
		let collider_radius = Units::from(11.);
		let animation = Some(Animation::new(
			AnimationAsset::from("my/asset"),
			PlayMode::Replay,
		));
		let mut app = setup();
		app.world_mut()
			.spawn((Transform::from_xyz(1., 2., 3.), Player));
		app.world_mut().spawn((
			Transform::from_xyz(1., 2., 7.1),
			Enemy {
				aggro_range: Units::from(4.),
				attack_range: Units::from(3.),
				min_target_distance: None,
			},
			MovementConfig {
				speed,
				collider_radius,
				animation: animation.clone(),
			},
			_Movement::new().with_mock(move |mock| {
				mock.expect_start::<Vec3>().never();
				mock.expect_start::<Dir3>().never();
				mock.expect_stop().once().return_const(());
				mock.expect_current_movement()
					.return_const(Some(MovementTarget::Point(Vec3::ONE)));
			}),
		));

		app.update();
	}

	#[test]
	fn do_not_chase_player_when_below_min_range() {
		let speed = UnitsPerSecond::from(42.);
		let collider_radius = Units::from(11.);
		let animation = Some(Animation::new(
			AnimationAsset::from("my/asset"),
			PlayMode::Replay,
		));
		let mut app = setup();
		app.world_mut()
			.spawn((Transform::from_xyz(1., 2., 3.), Player));
		app.world_mut().spawn((
			Transform::from_xyz(1., 2., 4.9),
			Enemy {
				aggro_range: Units::from(4.),
				attack_range: Units::from(3.),
				min_target_distance: Some(Units::from(2.)),
			},
			MovementConfig {
				speed,
				collider_radius,
				animation: animation.clone(),
			},
			_Movement::new().with_mock(move |mock| {
				mock.expect_start::<Vec3>().never();
				mock.expect_start::<Dir3>().never();
				mock.expect_stop().never();
				mock.expect_current_movement().return_const(None);
			}),
		));

		app.update();
	}

	#[test]
	fn stop_chase_player_when_below_min_range() {
		let speed = UnitsPerSecond::from(42.);
		let collider_radius = Units::from(11.);
		let animation = Some(Animation::new(
			AnimationAsset::from("my/asset"),
			PlayMode::Replay,
		));
		let mut app = setup();
		app.world_mut()
			.spawn((Transform::from_xyz(1., 2., 3.), Player));
		app.world_mut().spawn((
			Transform::from_xyz(1., 2., 4.9),
			Enemy {
				aggro_range: Units::from(4.),
				attack_range: Units::from(3.),
				min_target_distance: Some(Units::from(2.)),
			},
			MovementConfig {
				speed,
				collider_radius,
				animation: animation.clone(),
			},
			_Movement::new().with_mock(move |mock| {
				mock.expect_start::<Vec3>().never();
				mock.expect_start::<Dir3>().never();
				mock.expect_stop().once().return_const(());
				mock.expect_current_movement()
					.return_const(Some(MovementTarget::Point(Vec3::ONE)));
			}),
		));

		app.update();
	}

	#[test]
	fn do_not_chase_player_when_already_moving_to_player_position() {
		let speed = UnitsPerSecond::from(42.);
		let collider_radius = Units::from(11.);
		let animation = Some(Animation::new(
			AnimationAsset::from("my/asset"),
			PlayMode::Replay,
		));
		let mut app = setup();
		app.world_mut()
			.spawn((Transform::from_xyz(1., 2., 3.), Player));
		app.world_mut().spawn((
			Transform::from_xyz(1., 2., 6.9),
			Enemy {
				aggro_range: Units::from(4.),
				attack_range: Units::from(3.),
				min_target_distance: None,
			},
			MovementConfig {
				speed,
				collider_radius,
				animation: animation.clone(),
			},
			_Movement::new().with_mock(move |mock| {
				mock.expect_start::<Vec3>().never();
				mock.expect_start::<Dir3>().never();
				mock.expect_stop().never();
				mock.expect_current_movement()
					.return_const(MovementTarget::Point(Vec3::new(1., 2., 3.)));
			}),
		));

		app.update();
	}

	#[test]
	fn do_not_chase_player_when_in_aggro_range_but_attack_movement_is_attached() {
		let speed = UnitsPerSecond::from(42.);
		let collider_radius = Units::from(11.);
		let animation = Some(Animation::new(
			AnimationAsset::from("my/asset"),
			PlayMode::Replay,
		));
		let mut app = setup();
		app.world_mut()
			.spawn((Transform::from_xyz(1., 2., 3.), Player));
		app.world_mut().spawn((
			Transform::from_xyz(1., 2., 6.9),
			AttackMovement,
			Enemy {
				aggro_range: Units::from(4.),
				attack_range: Units::from(3.),
				min_target_distance: None,
			},
			MovementConfig {
				speed,
				collider_radius,
				animation: animation.clone(),
			},
			_Movement::new().with_mock(move |mock| {
				mock.expect_start::<Vec3>().never();
				mock.expect_start::<Dir3>().never();
				mock.expect_current_movement().return_const(None);
			}),
		));

		app.update();
	}
}
