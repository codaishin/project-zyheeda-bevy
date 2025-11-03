use crate::components::{enemy::Enemy, player::Player};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::traits::{
	accessors::get::EntityContextMut,
	handles_orientation::{FaceTargetIs, Facing, RegisterFaceTargetDefinition},
};

impl Player {
	pub(crate) fn register_target_definition<TFacing>(
		trigger: Trigger<OnAdd, Self>,
		enemies: Query<Entity, With<Enemy>>,
		mut facing: StaticSystemParam<TFacing>,
	) where
		TFacing: for<'c> EntityContextMut<Facing, TContext<'c>: RegisterFaceTargetDefinition>,
	{
		let player = trigger.target();

		for enemy in &enemies {
			let ctx = TFacing::get_entity_context_mut(&mut facing, enemy, Facing);
			let Some(mut ctx) = ctx else {
				continue;
			};
			ctx.register(FaceTargetIs::Entity(player));
		}

		let ctx = TFacing::get_entity_context_mut(&mut facing, player, Facing);
		let Some(mut ctx) = ctx else {
			return;
		};

		ctx.register(FaceTargetIs::Cursor);
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

		app.add_observer(Player::register_target_definition::<Query<&mut _Facing>>);

		app
	}

	#[test]
	fn add_cursor_facing() {
		let mut app = setup();

		app.world_mut().spawn((
			_Facing::new().with_mock(|mock| {
				mock.expect_register()
					.once()
					.with(eq(FaceTargetIs::Cursor))
					.return_const(());
			}),
			Player,
		));
	}

	#[test]
	fn add_player_facing_for_existing_enemies() {
		let mut app = setup();
		let player = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			_Facing::new().with_mock(move |mock| {
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

		app.world_mut().entity_mut(player).insert((
			_Facing::new().with_mock(|mock| {
				mock.expect_register().return_const(());
			}),
			Player,
		));
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();

		app.world_mut()
			.spawn(_Facing::new().with_mock(|mock| {
				mock.expect_register().once().return_const(());
			}))
			.insert(Player)
			.insert(Player);
	}
}
