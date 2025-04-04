use super::player::Player;
use bevy::prelude::*;
use common::{
	tools::{animation_key::AnimationKey, slot_key::SlotKey},
	traits::{
		animation::{Animation, AnimationPriority, PlayMode, StartAnimation, StopAnimation},
		try_remove_from::TryRemoveFrom,
	},
};

#[derive(Component, Debug, PartialEq, Clone, Copy)]
pub enum SkillAnimation {
	Start(SlotKey),
	Stop,
}

impl SkillAnimation {
	pub(crate) fn system<TAnimationDispatch>(
		mut commands: Commands,
		mut players: Query<
			(Entity, &SkillAnimation, &mut TAnimationDispatch),
			Added<SkillAnimation>,
		>,
	) where
		TAnimationDispatch: Component + StartAnimation + StopAnimation,
	{
		for (entity, apply, mut dispatch) in &mut players {
			match apply {
				SkillAnimation::Start(slot) => dispatch.start_animation(
					Skill,
					Animation::new(
						Player::animation_paths(AnimationKey::Other(*slot)),
						PlayMode::Repeat,
					),
				),
				SkillAnimation::Stop => {
					dispatch.stop_animation(Skill);
				}
			};
			commands.try_remove_from::<SkillAnimation>(entity);
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
		test_tools::utils::SingleThreadedApp,
		tools::slot_key::Side,
		traits::{
			animation::{Animation, AnimationPriority, StartAnimation},
			nested_mock::NestedMocks,
		},
	};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};
	use test_case::test_case;

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

	#[test_case(SlotKey::TopHand(Side::Left); "top left")]
	#[test_case(SlotKey::TopHand(Side::Right); "top right")]
	#[test_case(SlotKey::BottomHand(Side::Left); "bottom left")]
	#[test_case(SlotKey::BottomHand(Side::Right); "bottom right")]
	fn play_animation(slot: SlotKey) {
		let mut app = setup();
		app.world_mut().spawn((
			Player,
			SkillAnimation::Start(slot),
			_Dispatch::new().with_mock(|mock| {
				mock.expect_start_animation()
					.with(
						eq(Skill),
						eq(Animation::new(
							Player::animation_paths(AnimationKey::Other(slot)),
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
			SkillAnimation::Start(SlotKey::TopHand(Side::Left)),
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
				SkillAnimation::Start(SlotKey::TopHand(Side::Left)),
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
