use crate::components::{enemy::Enemy, player::Player};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::traits::{
	accessors::get::GetContextMut,
	handles_orientation::{FaceTargetIs, Facing, RegisterFaceTargetDefinition},
};

impl Enemy {
	pub(crate) fn register_target_definition<TFacing>(
		trigger: Trigger<OnAdd, Self>,
		players: Query<Entity, With<Player>>,
		mut facing: StaticSystemParam<TFacing>,
	) where
		TFacing: for<'c> GetContextMut<Facing, TContext<'c>: RegisterFaceTargetDefinition>,
	{
		let entity = trigger.target();
		let Ok(player) = players.single() else {
			return;
		};
		let ctx = TFacing::get_context_mut(&mut facing, Facing { entity });
		let Some(mut ctx) = ctx else {
			return;
		};

		ctx.register(FaceTargetIs::Entity(player));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::enemy::Enemy;
	use common::{tools::Units, traits::handles_orientation::FaceTargetIs};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component, NestedMocks)]
	struct _Facing {
		mock: Mock_Facing,
	}

	#[automock]
	impl RegisterFaceTargetDefinition for _Facing {
		fn register(&mut self, face_target_is: FaceTargetIs) {
			self.mock.register(face_target_is);
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(Enemy::register_target_definition::<Query<&mut _Facing>>);

		app
	}

	#[test]
	fn add_player_facing() {
		let mut app = setup();
		let player = app.world_mut().spawn(Player).id();

		app.world_mut().spawn((
			_Facing::new().with_mock(|mock| {
				mock.expect_register()
					.once()
					.with(eq(FaceTargetIs::Entity(player)))
					.return_const(());
			}),
			Enemy {
				aggro_range: Units::from(11.),
				attack_range: Units::from(5.),
				min_target_distance: None,
			},
		));
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		app.world_mut().spawn(Player);

		app.world_mut()
			.spawn(_Facing::new().with_mock(|mock| {
				mock.expect_register().once().return_const(());
			}))
			.insert(Enemy {
				aggro_range: Units::from(11.),
				attack_range: Units::from(5.),
				min_target_distance: None,
			})
			.insert(Enemy {
				aggro_range: Units::from(11.),
				attack_range: Units::from(5.),
				min_target_distance: None,
			});
	}
}
