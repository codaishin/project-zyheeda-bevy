use crate::components::{
	enemy::{Enemy, chasing::Chasing},
	movement_config::MovementConfig,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::traits::{
	accessors::get::GetContextMut,
	handles_movement::{CurrentMovement, Movement, MovementTarget, StartMovement, StopMovement},
};

impl Enemy {
	pub(crate) fn chase_player<TMovement>(
		mut movement: StaticSystemParam<TMovement>,
		enemies: Query<(Entity, &MovementConfig, Option<&Chasing>), With<Self>>,
		transforms: Query<&Transform>,
	) where
		TMovement: for<'c> GetContextMut<
				Movement,
				TContext<'c>: StartMovement + StopMovement + CurrentMovement,
			>,
	{
		for (entity, config, chasing) in &enemies {
			let ctx = TMovement::get_context_mut(&mut movement, Movement { entity });
			let Some(mut ctx) = ctx else {
				continue;
			};

			match (chasing, ctx.current_movement()) {
				(None, Some(_)) => {
					ctx.stop();
				}
				(Some(Chasing { player }), current_movement) => {
					let Ok(player) = transforms.get(*player) else {
						continue;
					};
					if current_movement == Some(MovementTarget::Point(player.translation)) {
						continue;
					}
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
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::movement_config::MovementConfig;
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
	#[test_case(Some(MovementTarget::Dir(Dir3::NEG_Z)); "and current movement is directional")]
	#[test_case(None; "and no current movement")]
	fn move_to_chase_player(current_movement: Option<MovementTarget>) {
		let speed = UnitsPerSecond::from(42.);
		let collider_radius = Units::from(11.);
		let animation = Some(Animation::new(
			AnimationAsset::from("my/asset"),
			PlayMode::Replay,
		));
		let mut app = setup();
		let player = app.world_mut().spawn(Transform::from_xyz(1., 2., 3.)).id();
		app.world_mut().spawn((
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
			Chasing { player },
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
	fn stop_moving_when_not_chasing() {
		let speed = UnitsPerSecond::from(42.);
		let collider_radius = Units::from(11.);
		let animation = Some(Animation::new(
			AnimationAsset::from("my/asset"),
			PlayMode::Replay,
		));
		let mut app = setup();
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
	fn do_not_move_when_chasing_and_already_moving_to_same_place() {
		let speed = UnitsPerSecond::from(42.);
		let collider_radius = Units::from(11.);
		let animation = Some(Animation::new(
			AnimationAsset::from("my/asset"),
			PlayMode::Replay,
		));
		let mut app = setup();
		let player = app.world_mut().spawn(Transform::from_xyz(1., 2., 3.)).id();
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
			Chasing { player },
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
	fn do_not_stop_moving_when_not_chasing_but_not_already_moving() {
		let speed = UnitsPerSecond::from(42.);
		let collider_radius = Units::from(11.);
		let animation = Some(Animation::new(
			AnimationAsset::from("my/asset"),
			PlayMode::Replay,
		));
		let mut app = setup();
		app.world_mut().spawn((
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
}
