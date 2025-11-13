use crate::components::player::PlayerAnimationKey;

use super::player::Player;
use bevy::{ecs::component::Mutable, prelude::*};
use common::{
	tools::action_key::slot::PlayerSlot,
	traits::{
		accessors::get::TryApplyOn,
		animation::{Animation, AnimationPriority, PlayMode, StartAnimation, StopAnimation},
	},
	zyheeda_commands::ZyheedaCommands,
};

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub enum SkillAnimation {
	Start(PlayerSlot),
	Stop,
}

impl SkillAnimation {
	pub(crate) fn system<TAnimationDispatch>(
		mut commands: ZyheedaCommands,
		mut players: Query<
			(Entity, &SkillAnimation, &mut TAnimationDispatch),
			Added<SkillAnimation>,
		>,
	) where
		TAnimationDispatch: Component<Mutability = Mutable> + StartAnimation + StopAnimation,
	{
		for (entity, apply, mut dispatch) in &mut players {
			match apply {
				SkillAnimation::Start(slot) => dispatch.start_animation(
					Skill,
					Animation::new(
						Player::animation_asset(PlayerAnimationKey::Skill(*slot)),
						PlayMode::Repeat,
					),
				),
				SkillAnimation::Stop => {
					dispatch.stop_animation(Skill);
				}
			};
			commands.try_apply_on(&entity, |mut e| {
				e.try_remove::<SkillAnimation>();
			});
		}
	}
}

#[derive(Debug, PartialEq)]
struct Skill;

impl From<Skill> for AnimationPriority {
	fn from(_: Skill) -> Self {
		AnimationPriority::High
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::player::Player;
	use common::{
		tools::action_key::slot::Side,
		traits::animation::{Animation, AnimationPriority, StartAnimation},
	};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};
	use test_case::test_case;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component, NestedMocks)]
	struct _Dispatch {
		mock: Mock_Dispatch,
	}

	impl StartAnimation for _Dispatch {
		fn start_animation<TLayer>(&mut self, layer: TLayer, animation: Animation)
		where
			TLayer: Into<AnimationPriority> + 'static,
		{
			self.mock.start_animation(layer, animation);
		}
	}

	impl StopAnimation for _Dispatch {
		fn stop_animation<TLayer>(&mut self, layer: TLayer)
		where
			TLayer: Into<AnimationPriority> + 'static,
		{
			self.mock.stop_animation(layer);
		}
	}

	mock! {
		_Dispatch {}
		impl StartAnimation for _Dispatch {
			fn start_animation<TLayer>(&mut self, layer: TLayer, animation: Animation)
			where
				TLayer: Into<AnimationPriority> + 'static;
		}
		impl StopAnimation for _Dispatch {
			fn stop_animation<TLayer>(&mut self, layer: TLayer)
			where
				TLayer: Into<AnimationPriority> + 'static;
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, SkillAnimation::system::<_Dispatch>);

		app
	}

	#[test_case(PlayerSlot::Upper(Side::Left); "top left")]
	#[test_case(PlayerSlot::Upper(Side::Right); "top right")]
	#[test_case(PlayerSlot::Lower(Side::Left); "bottom left")]
	#[test_case(PlayerSlot::Lower(Side::Right); "bottom right")]
	fn play_animation(slot: PlayerSlot) {
		let mut app = setup();
		app.world_mut().spawn((
			Player,
			SkillAnimation::Start(slot),
			_Dispatch::new().with_mock(|mock| {
				mock.expect_start_animation()
					.with(
						eq(Skill),
						eq(Animation::new(
							Player::animation_asset(PlayerAnimationKey::Skill(slot)),
							PlayMode::Repeat,
						)),
					)
					.times(1)
					.return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn stop_animation() {
		let mut app = setup();
		app.world_mut().spawn((
			Player,
			SkillAnimation::Stop,
			_Dispatch::new().with_mock(|mock| {
				mock.expect_stop_animation()
					.with(eq(Skill))
					.times(1)
					.return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn only_operate_when_skill_animation_is_added() {
		let mut app = setup();
		app.world_mut().spawn((
			Player,
			SkillAnimation::Start(PlayerSlot::Upper(Side::Left)),
			_Dispatch::new().with_mock(|mock| {
				mock.expect_start_animation::<Skill>()
					.times(1)
					.return_const(());
			}),
		));

		app.update();
		app.update();
	}

	#[test]
	fn remove_skill_animation() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Player,
				SkillAnimation::Start(PlayerSlot::Upper(Side::Left)),
				_Dispatch::new().with_mock(|mock| {
					mock.expect_start_animation::<Skill>()
						.times(1)
						.return_const(());
				}),
			))
			.id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<SkillAnimation>());
	}
}
