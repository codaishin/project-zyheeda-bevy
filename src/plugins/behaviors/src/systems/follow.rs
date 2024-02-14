use crate::traits::MoveTogether;
use bevy::prelude::{Component, Query, Transform, With, Without};

pub(crate) fn follow<TTarget: Component, TMover: MoveTogether + Component>(
	target: Query<(&Transform, With<TTarget>)>,
	mut follower: Query<(&mut Transform, &mut TMover, Without<TTarget>)>,
) {
	let Ok((target, ..)) = target.get_single() else {
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
	use mockall::{automock, predicate::eq};

	#[derive(Component)]
	struct _Target;

	#[derive(Component)]
	struct _Mover {
		pub mock: Mock_Mover,
	}

	impl _Mover {
		fn new() -> Self {
			Self {
				mock: Mock_Mover::new(),
			}
		}
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
		let target = Vec3::new(1., 2., 3.);
		let follow_transform = Transform::from_xyz(10., 10., 10.);
		let mut mover = _Mover::new();

		mover
			.mock
			.expect_move_together_with()
			.with(eq(follow_transform), eq(target))
			.times(1)
			.return_const(());

		app.world
			.spawn((_Target, Transform::from_translation(target)));
		app.world.spawn((mover, follow_transform));
		app.add_systems(Update, follow::<_Target, _Mover>);

		app.update();
	}
}
