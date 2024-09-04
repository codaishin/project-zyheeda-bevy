use crate::traits::MoveTogether;
use bevy::prelude::{Component, Query, Transform, With, Without};

pub(crate) fn follow<TTarget: Component, TMover: MoveTogether + Component>(
	targets: Query<&Transform, With<TTarget>>,
	mut follower: Query<(&mut Transform, &mut TMover), Without<TTarget>>,
) {
	let Ok(target) = targets.get_single() else {
		return; //FIXME: Handle properly;
	};
	let Ok((mut follower, mut mover, ..)) = follower.get_single_mut() else {
		return; //FIXME: Handle properly;
	};

	mover.move_together_with(&mut follower, target.translation);
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::{App, Component, Transform, Update, Vec3};
	use common::traits::nested_mock::NestedMocks;
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};

	#[derive(Component)]
	struct _Target;

	#[derive(Component, NestedMocks)]
	struct _Mover {
		pub mock: Mock_Mover,
	}

	#[automock]
	impl MoveTogether for _Mover {
		fn move_together_with(&mut self, transform: &mut Transform, new_position: Vec3) {
			self.mock.move_together_with(transform, new_position)
		}
	}

	#[test]
	fn do_follow() {
		let mut app = App::new();
		app.add_systems(Update, follow::<_Target, _Mover>);
		app.world_mut()
			.spawn((_Target, Transform::from_translation(Vec3::new(1., 2., 3.))));
		app.world_mut().spawn((
			_Mover::new().with_mock(|mock| {
				mock.expect_move_together_with()
					.with(
						eq(Transform::from_xyz(10., 10., 10.)),
						eq(Vec3::new(1., 2., 3.)),
					)
					.times(1)
					.return_const(());
			}),
			Transform::from_xyz(10., 10., 10.),
		));

		app.update();
	}
}
