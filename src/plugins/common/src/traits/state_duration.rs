use crate::traits::iteration::IterKey;
use std::{
	cmp::Ordering,
	collections::HashSet,
	hash::Hash,
	ops::{Add, Sub},
	time::Duration,
};

#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum StateMeta<TStateName: Clone> {
	First,
	In(TStateName),
	Leaving(TStateName),
}

pub trait StateDuration<TStateKey> {
	fn get_state_duration(&self, key: TStateKey) -> Duration;
	fn elapsed_mut(&mut self) -> &mut Duration;
}

pub trait StateUpdate<T: Clone> {
	fn update_state(&mut self, delta: Duration) -> HashSet<StateMeta<T>>;
}

/// Duration clamped at MAX (represented as Infinite) and ZERO
#[derive(PartialEq, Clone, Copy)]
enum _SafeDuration {
	Infinite,
	Finite(Duration),
}

impl _SafeDuration {
	const ZERO: _SafeDuration = _SafeDuration::Finite(Duration::ZERO);
}

impl PartialOrd for _SafeDuration {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		match (self, other) {
			(_SafeDuration::Finite(a), _SafeDuration::Finite(b)) => a.partial_cmp(b),
			(_SafeDuration::Infinite, _SafeDuration::Infinite) => Some(Ordering::Equal),
			(_SafeDuration::Infinite, _) => Some(Ordering::Greater),
			(_, _SafeDuration::Infinite) => Some(Ordering::Less),
		}
	}
}

impl From<Duration> for _SafeDuration {
	fn from(value: Duration) -> Self {
		match value {
			Duration::MAX => _SafeDuration::Infinite,
			value => _SafeDuration::Finite(value),
		}
	}
}

impl From<_SafeDuration> for Duration {
	fn from(value: _SafeDuration) -> Self {
		match value {
			_SafeDuration::Infinite => Duration::MAX,
			_SafeDuration::Finite(value) => value,
		}
	}
}

impl Add for _SafeDuration {
	type Output = _SafeDuration;

	fn add(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(_SafeDuration::Finite(a), _SafeDuration::Finite(b)) => a
				.checked_add(b)
				.map_or(_SafeDuration::Infinite, _SafeDuration::Finite),
			_ => _SafeDuration::Infinite,
		}
	}
}

impl Sub for _SafeDuration {
	type Output = _SafeDuration;

	fn sub(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(_SafeDuration::Finite(a), _SafeDuration::Finite(b)) => {
				_SafeDuration::from(a.checked_sub(b).unwrap_or_default())
			}
			(_SafeDuration::Infinite, _) => _SafeDuration::Infinite,
			_ => _SafeDuration::ZERO,
		}
	}
}

impl<
		TStateDuration: StateDuration<TStateKey>,
		TStateKey: IterKey + Copy + Clone + Eq + Hash + 'static,
	> StateUpdate<TStateKey> for TStateDuration
{
	fn update_state(&mut self, delta: Duration) -> HashSet<StateMeta<TStateKey>> {
		let state_keys = TStateKey::iterator();
		let mut states = HashSet::new();
		let mut state_end = _SafeDuration::ZERO;
		let before_update = _SafeDuration::from(*self.elapsed_mut());
		let after_update = before_update + delta.into();

		if before_update == _SafeDuration::ZERO {
			states.insert(StateMeta::First);
		}

		for state_key in state_keys {
			let state_begin = state_end;

			state_end = state_end + self.get_state_duration(state_key).into();
			for meta in current_state_metas(before_update, after_update, state_begin, state_end) {
				states.insert(meta(state_key));
			}
		}

		*self.elapsed_mut() = after_update.into();

		states
	}
}

fn current_state_metas<TStateName: IterKey + Copy + Clone + Eq + Hash + 'static>(
	before_update: _SafeDuration,
	after_update: _SafeDuration,
	state_begin: _SafeDuration,
	state_end: _SafeDuration,
) -> Vec<Box<dyn Fn(TStateName) -> StateMeta<TStateName>>> {
	match state_end - state_begin > _SafeDuration::ZERO {
		true => non_zero_duration_meta(before_update, after_update, state_begin, state_end),
		false => zero_duration_meta(before_update, after_update, state_begin),
	}
}

fn non_zero_duration_meta<TStateName: IterKey + Copy + Clone + Eq + Hash + 'static>(
	before_update: _SafeDuration,
	after_update: _SafeDuration,
	state_begin: _SafeDuration,
	state_end: _SafeDuration,
) -> Vec<Box<dyn Fn(TStateName) -> StateMeta<TStateName>>> {
	if after_update < state_begin {
		vec![]
	} else if after_update < state_end {
		vec![Box::new(StateMeta::In)]
	} else if before_update < state_end {
		vec![Box::new(StateMeta::In), Box::new(StateMeta::Leaving)]
	} else {
		vec![]
	}
}

fn zero_duration_meta<TStateName: IterKey + Copy + Clone + Eq + Hash + 'static>(
	before_update: _SafeDuration,
	after_update: _SafeDuration,
	current_state: _SafeDuration,
) -> Vec<Box<dyn Fn(TStateName) -> StateMeta<TStateName>>> {
	if before_update <= current_state || after_update >= current_state {
		vec![Box::new(StateMeta::In), Box::new(StateMeta::Leaving)]
	} else {
		vec![]
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::iteration::Iter;

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	enum _State {
		A,
		B,
		C,
	}

	#[derive(Default)]
	struct _Agent {
		pub state_a: Duration,
		pub state_b: Duration,
		pub state_c: Duration,
		pub duration: Duration,
	}

	impl IterKey for _State {
		fn iterator() -> Iter<Self> {
			Iter(Some(_State::A))
		}

		fn next(current: &Iter<Self>) -> Option<Self> {
			match current.0? {
				_State::A => Some(_State::B),
				_State::B => Some(_State::C),
				_State::C => None,
			}
		}
	}

	impl StateDuration<_State> for _Agent {
		fn elapsed_mut(&mut self) -> &mut Duration {
			&mut self.duration
		}

		fn get_state_duration(&self, state_name: _State) -> Duration {
			match state_name {
				_State::A => self.state_a,
				_State::B => self.state_b,
				_State::C => self.state_c,
			}
		}
	}

	#[test]
	fn update_duration() {
		let mut agent = _Agent::default();

		agent.update_state(Duration::from_millis(11));
		agent.update_state(Duration::from_millis(15));
		agent.update_state(Duration::from_millis(16));

		assert_eq!(Duration::from_millis(42), agent.duration);
	}

	#[test]
	fn get_only_a() {
		let mut agent = _Agent {
			state_a: Duration::from_millis(10),
			state_b: Duration::from_millis(20),
			state_c: Duration::from_millis(30),
			duration: Duration::ZERO,
		};

		assert_eq!(
			HashSet::from([StateMeta::First, StateMeta::In(_State::A),]),
			agent.update_state(Duration::from_millis(9))
		);
	}

	#[test]
	fn get_exit_a() {
		let mut agent = _Agent {
			state_a: Duration::from_millis(10),
			state_b: Duration::from_millis(20),
			state_c: Duration::from_millis(30),
			duration: Duration::ZERO,
		};

		assert_eq!(
			HashSet::from([
				StateMeta::First,
				StateMeta::In(_State::A),
				StateMeta::Leaving(_State::A),
				StateMeta::In(_State::B),
			]),
			agent.update_state(Duration::from_millis(11))
		);
	}

	#[test]
	fn get_only_b() {
		let mut agent = _Agent {
			state_a: Duration::from_millis(10),
			state_b: Duration::from_millis(20),
			state_c: Duration::from_millis(30),
			duration: Duration::from_millis(11),
		};

		assert_eq!(
			HashSet::from([StateMeta::In(_State::B)]),
			agent.update_state(Duration::from_millis(2))
		);
	}

	#[test]
	fn get_exit_b() {
		let mut agent = _Agent {
			state_a: Duration::from_millis(10),
			state_b: Duration::from_millis(20),
			state_c: Duration::from_millis(30),
			duration: Duration::from_millis(15),
		};

		assert_eq!(
			HashSet::from([
				StateMeta::In(_State::B),
				StateMeta::Leaving(_State::B),
				StateMeta::In(_State::C)
			]),
			agent.update_state(Duration::from_millis(16))
		);
	}

	#[test]
	fn get_all_states() {
		let mut agent = _Agent {
			state_a: Duration::from_millis(1),
			state_b: Duration::from_millis(2),
			state_c: Duration::from_millis(3),
			duration: Duration::ZERO,
		};

		assert_eq!(
			HashSet::from([
				StateMeta::First,
				StateMeta::In(_State::A),
				StateMeta::Leaving(_State::A),
				StateMeta::In(_State::B),
				StateMeta::Leaving(_State::B),
				StateMeta::In(_State::C),
				StateMeta::Leaving(_State::C),
			]),
			agent.update_state(Duration::from_millis(10))
		);
	}

	#[test]
	fn get_all_states_with_all_zero_time() {
		let mut agent = _Agent {
			state_a: Duration::ZERO,
			state_b: Duration::ZERO,
			state_c: Duration::ZERO,
			duration: Duration::ZERO,
		};

		assert_eq!(
			HashSet::from([
				StateMeta::First,
				StateMeta::In(_State::A),
				StateMeta::Leaving(_State::A),
				StateMeta::In(_State::B),
				StateMeta::Leaving(_State::B),
				StateMeta::In(_State::C),
				StateMeta::Leaving(_State::C),
			]),
			agent.update_state(Duration::from_millis(10))
		);
	}

	#[test]
	fn get_single_state_with_all_zero_time() {
		let mut agent = _Agent {
			state_a: Duration::from_secs(4),
			state_b: Duration::ZERO,
			state_c: Duration::from_secs(4),
			duration: Duration::ZERO,
		};

		assert_eq!(
			HashSet::from([
				StateMeta::First,
				StateMeta::In(_State::A),
				StateMeta::Leaving(_State::A),
				StateMeta::In(_State::B),
				StateMeta::Leaving(_State::B),
				StateMeta::In(_State::C),
			]),
			agent.update_state(Duration::from_secs(7))
		);
	}

	#[test]
	fn can_handle_infinite_durations() {
		let mut agent = _Agent {
			state_a: Duration::MAX,
			state_b: Duration::from_secs(1),
			state_c: Duration::from_secs(1),
			duration: Duration::ZERO,
		};

		assert_eq!(
			HashSet::from([StateMeta::First, StateMeta::In(_State::A),]),
			agent.update_state(Duration::from_secs(1))
		);
	}

	#[test]
	fn get_all_progressively() {
		let mut agent = _Agent {
			state_a: Duration::from_secs(2),
			state_b: Duration::from_secs(2),
			state_c: Duration::from_secs(2),
			duration: Duration::ZERO,
		};

		let states = [
			agent.update_state(Duration::from_secs(1)),
			agent.update_state(Duration::from_secs(1)),
			agent.update_state(Duration::from_secs(1)),
			agent.update_state(Duration::from_secs(1)),
			agent.update_state(Duration::from_secs(1)),
			agent.update_state(Duration::from_secs(1)),
		];

		assert_eq!(
			[
				HashSet::from([StateMeta::First, StateMeta::In(_State::A),]),
				HashSet::from([
					StateMeta::In(_State::A),
					StateMeta::Leaving(_State::A),
					StateMeta::In(_State::B),
				]),
				HashSet::from([StateMeta::In(_State::B),]),
				HashSet::from([
					StateMeta::In(_State::B),
					StateMeta::Leaving(_State::B),
					StateMeta::In(_State::C),
				]),
				HashSet::from([StateMeta::In(_State::C),]),
				HashSet::from([StateMeta::In(_State::C), StateMeta::Leaving(_State::C),])
			],
			states
		);
	}
}
