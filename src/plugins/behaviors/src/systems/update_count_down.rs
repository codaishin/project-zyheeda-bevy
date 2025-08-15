use crate::traits::count_down::CountDown;
use bevy::{ecs::component::Mutable, prelude::*};
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};
use std::time::Duration;

impl<T> UpdateCountDown for T where T: CountDown + Component<Mutability = Mutable> {}

pub(crate) trait UpdateCountDown: CountDown + Component<Mutability = Mutable> {
	fn update<TTime: Default + Send + Sync + 'static>(
		mut commands: ZyheedaCommands,
		mut cool_downs: Query<(Entity, &mut Self)>,
		time: Res<Time<TTime>>,
	) {
		if cool_downs.is_empty() {
			return;
		}

		let delta = time.delta();

		for (entity, mut count_down) in &mut cool_downs {
			if *count_down.remaining_mut() <= delta {
				match get_next(count_down.as_mut(), delta) {
					Some(next) => {
						*count_down = next;
					}
					None => {
						commands.try_apply_on(&entity, |mut e| {
							e.try_remove::<Self>();
						});
					}
				}
			} else {
				*count_down.remaining_mut() -= delta;
			}
		}
	}
}

fn get_next<T>(count_down: &mut T, delta: Duration) -> Option<T>
where
	T: CountDown + Component,
{
	let mut next_state = count_down.next_state()?;
	let next_remaining = next_state.remaining_mut();
	let rest = delta - *count_down.remaining_mut();
	if rest >= *next_remaining {
		*next_remaining = Duration::ZERO;
	} else {
		*next_remaining -= rest;
	}
	Some(next_state)
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		time::Real,
	};
	use std::time::Duration;
	use testing::{SingleThreadedApp, TickTime};

	#[derive(Component, Debug, PartialEq)]
	enum _CountDown {
		A { remaining: Duration, b: Duration },
		B(Duration),
	}

	impl CountDown for _CountDown {
		fn remaining_mut(&mut self) -> &mut Duration {
			match self {
				_CountDown::A { remaining, .. } => remaining,
				_CountDown::B(duration) => duration,
			}
		}

		fn next_state(&self) -> Option<Self> {
			match self {
				_CountDown::A { b, .. } => Some(Self::B(*b)),
				_CountDown::B(_) => None,
			}
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, _CountDown::update::<Real>);
		app.init_resource::<Time<Real>>();

		app
	}

	#[test]
	fn reduce_by_delta() {
		let mut app = setup();
		let cool_down = app
			.world_mut()
			.spawn(_CountDown::A {
				remaining: Duration::from_millis(1000),
				b: Duration::default(),
			})
			.id();

		app.tick_time(Duration::from_millis(42));
		app.update();

		let cool_down = app.world().entity(cool_down);

		assert_eq!(
			Some(&_CountDown::A {
				remaining: Duration::from_millis(958),
				b: Duration::default(),
			}),
			cool_down.get::<_CountDown>()
		);
	}

	#[test]
	fn insert_next_if_remaining_cool_down_is_zero() {
		let mut app = setup();
		let cool_down = app
			.world_mut()
			.spawn(_CountDown::A {
				remaining: Duration::from_millis(42),
				b: Duration::from_millis(1000),
			})
			.id();

		app.tick_time(Duration::from_millis(42));
		app.update();

		let cool_down = app.world().entity(cool_down);

		assert_eq!(
			Some(&_CountDown::B(Duration::from_millis(1000))),
			cool_down.get::<_CountDown>()
		);
	}

	#[test]
	fn insert_next_if_remaining_cool_down_is_negative() {
		let mut app = setup();
		let cool_down = app
			.world_mut()
			.spawn(_CountDown::A {
				remaining: Duration::from_millis(10),
				b: Duration::from_millis(1000),
			})
			.id();

		app.tick_time(Duration::from_millis(42));
		app.update();

		let cool_down = app.world().entity(cool_down);

		assert_eq!(
			Some(&_CountDown::B(Duration::from_millis(968))),
			cool_down.get::<_CountDown>()
		);
	}

	#[test]
	fn insert_next_with_zero_remaining_if_it_would_have_negative_remaining() {
		let mut app = setup();
		let cool_down = app
			.world_mut()
			.spawn(_CountDown::A {
				remaining: Duration::from_millis(10),
				b: Duration::from_millis(10),
			})
			.id();

		app.tick_time(Duration::from_millis(42));
		app.update();

		let cool_down = app.world().entity(cool_down);

		assert_eq!(
			Some(&_CountDown::B(Duration::from_millis(0))),
			cool_down.get::<_CountDown>()
		);
	}

	#[test]
	fn remove_if_remaining_cool_down_is_zero_and_next_is_none() {
		let mut app = setup();
		let cool_down = app
			.world_mut()
			.spawn(_CountDown::B(Duration::from_millis(42)))
			.id();

		app.tick_time(Duration::from_millis(42));
		app.update();

		let cool_down = app.world().entity(cool_down);

		assert_eq!(None, cool_down.get::<_CountDown>());
	}

	#[test]
	fn remove_if_remaining_cool_down_is_negative_and_next_is_none() {
		let mut app = setup();
		let cool_down = app
			.world_mut()
			.spawn(_CountDown::B(Duration::from_millis(10)))
			.id();

		app.tick_time(Duration::from_millis(42));
		app.update();

		let cool_down = app.world().entity(cool_down);

		assert_eq!(None, cool_down.get::<_CountDown>());
	}
}
