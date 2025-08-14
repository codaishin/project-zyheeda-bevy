use crate::traits::iteration::IterFinite;
use std::{
	cmp::Ordering,
	collections::HashSet,
	hash::Hash,
	ops::{Add, Sub},
	time::Duration,
};

#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub enum StateMeta<TStateName> {
	Entering(TStateName),
	In(TStateName),
	Done,
}

pub trait StateDuration<TStateKey> {
	fn get_state_duration(&self, key: TStateKey) -> Duration;
	fn elapsed(&self) -> Duration;
	fn set_elapsed(&mut self, new_duration: Duration);
}

pub trait UpdatedStates<T> {
	fn updated_states(&mut self, delta: Duration) -> HashSet<StateMeta<T>>;
}

/// Duration clamped at MAX (represented as Infinite) and ZERO
#[derive(PartialEq, Clone, Copy)]
enum SafeDuration {
	Infinite,
	Finite(Duration),
}

impl SafeDuration {
	const ZERO: SafeDuration = SafeDuration::Finite(Duration::ZERO);
}

impl PartialOrd for SafeDuration {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		match (self, other) {
			(SafeDuration::Finite(a), SafeDuration::Finite(b)) => a.partial_cmp(b),
			(SafeDuration::Infinite, SafeDuration::Infinite) => Some(Ordering::Equal),
			(SafeDuration::Infinite, _) => Some(Ordering::Greater),
			(_, SafeDuration::Infinite) => Some(Ordering::Less),
		}
	}
}

impl From<Duration> for SafeDuration {
	fn from(value: Duration) -> Self {
		match value {
			Duration::MAX => SafeDuration::Infinite,
			value => SafeDuration::Finite(value),
		}
	}
}

impl From<SafeDuration> for Duration {
	fn from(value: SafeDuration) -> Self {
		match value {
			SafeDuration::Infinite => Duration::MAX,
			SafeDuration::Finite(value) => value,
		}
	}
}

impl Add for SafeDuration {
	type Output = SafeDuration;

	fn add(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(SafeDuration::Finite(a), SafeDuration::Finite(b)) => a
				.checked_add(b)
				.map_or(SafeDuration::Infinite, SafeDuration::Finite),
			_ => SafeDuration::Infinite,
		}
	}
}

impl Sub for SafeDuration {
	type Output = SafeDuration;

	fn sub(self, rhs: Self) -> Self::Output {
		match (self, rhs) {
			(SafeDuration::Finite(a), SafeDuration::Finite(b)) => {
				SafeDuration::from(a.checked_sub(b).unwrap_or_default())
			}
			(SafeDuration::Infinite, _) => SafeDuration::Infinite,
			_ => SafeDuration::ZERO,
		}
	}
}

impl<
	TStateDuration: StateDuration<TStateKey>,
	TStateKey: IterFinite + Copy + Clone + Eq + Hash + 'static,
> UpdatedStates<TStateKey> for TStateDuration
{
	fn updated_states(&mut self, delta: Duration) -> HashSet<StateMeta<TStateKey>> {
		let mut states = HashSet::new();
		let mut state_end = SafeDuration::ZERO;
		let before_update = SafeDuration::from(self.elapsed());
		let after_update = before_update + SafeDuration::from(delta);

		for state_key in TStateKey::iterator() {
			let state_begin = state_end;

			state_end = state_end + SafeDuration::from(self.get_state_duration(state_key));
			for meta in current_state_metas(before_update, after_update, state_begin, state_end) {
				states.insert(meta(state_key));
			}
		}

		if after_update > state_end {
			states.insert(StateMeta::Done);
		}

		self.set_elapsed(after_update.into());

		states
	}
}

fn current_state_metas<TStateName: IterFinite + Copy + Clone + Eq + Hash + 'static>(
	before_update: SafeDuration,
	after_update: SafeDuration,
	state_begin: SafeDuration,
	state_end: SafeDuration,
) -> Vec<Box<dyn Fn(TStateName) -> StateMeta<TStateName>>> {
	match state_end - state_begin > SafeDuration::ZERO {
		true => non_zero_duration_meta(before_update, after_update, state_begin, state_end),
		false => zero_duration_meta(before_update, after_update, state_begin),
	}
}

fn non_zero_duration_meta<TStateName: IterFinite + Copy + Clone + Eq + Hash + 'static>(
	before_update: SafeDuration,
	after_update: SafeDuration,
	state_begin: SafeDuration,
	state_end: SafeDuration,
) -> Vec<Box<dyn Fn(TStateName) -> StateMeta<TStateName>>> {
	if after_update <= state_begin {
		vec![]
	} else if before_update <= state_begin {
		vec![Box::new(StateMeta::Entering), Box::new(StateMeta::In)]
	} else if before_update < state_end {
		vec![Box::new(StateMeta::In)]
	} else {
		vec![]
	}
}

fn zero_duration_meta<TStateName: IterFinite + Copy + Clone + Eq + Hash + 'static>(
	before_update: SafeDuration,
	after_update: SafeDuration,
	current_state: SafeDuration,
) -> Vec<Box<dyn Fn(TStateName) -> StateMeta<TStateName>>> {
	if before_update <= current_state && current_state <= after_update {
		vec![Box::new(StateMeta::Entering), Box::new(StateMeta::In)]
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
		pub elapsed: Duration,
	}

	impl IterFinite for _State {
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
		fn set_elapsed(&mut self, new_duration: Duration) {
			self.elapsed = new_duration;
		}

		fn elapsed(&self) -> Duration {
			self.elapsed
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

		agent.updated_states(Duration::from_millis(11));
		agent.updated_states(Duration::from_millis(15));
		agent.updated_states(Duration::from_millis(16));

		assert_eq!(Duration::from_millis(42), agent.elapsed);
	}

	#[test]
	fn get_only_a() {
		let mut agent = _Agent {
			state_a: Duration::from_millis(10),
			state_b: Duration::from_millis(20),
			state_c: Duration::from_millis(30),
			elapsed: Duration::ZERO,
		};

		assert_eq!(
			HashSet::from([StateMeta::Entering(_State::A), StateMeta::In(_State::A)]),
			agent.updated_states(Duration::from_millis(9))
		);
	}

	#[test]
	fn get_exit_a() {
		let mut agent = _Agent {
			state_a: Duration::from_millis(10),
			state_b: Duration::from_millis(20),
			state_c: Duration::from_millis(30),
			elapsed: Duration::from_millis(1),
		};

		assert_eq!(
			HashSet::from([
				StateMeta::In(_State::A),
				StateMeta::Entering(_State::B),
				StateMeta::In(_State::B),
			]),
			agent.updated_states(Duration::from_millis(10))
		);
	}

	#[test]
	fn get_only_b() {
		let mut agent = _Agent {
			state_a: Duration::from_millis(10),
			state_b: Duration::from_millis(20),
			state_c: Duration::from_millis(30),
			elapsed: Duration::from_millis(11),
		};

		assert_eq!(
			HashSet::from([StateMeta::In(_State::B)]),
			agent.updated_states(Duration::from_millis(2))
		);
	}

	#[test]
	fn get_enter_b() {
		let mut agent = _Agent {
			state_a: Duration::from_millis(10),
			state_b: Duration::from_millis(20),
			state_c: Duration::from_millis(30),
			elapsed: Duration::from_millis(5),
		};

		assert_eq!(
			HashSet::from([
				StateMeta::In(_State::A),
				StateMeta::Entering(_State::B),
				StateMeta::In(_State::B)
			]),
			agent.updated_states(Duration::from_millis(10))
		);
	}

	#[test]
	fn get_all_states() {
		let mut agent = _Agent {
			state_a: Duration::from_millis(1),
			state_b: Duration::from_millis(2),
			state_c: Duration::from_millis(3),
			elapsed: Duration::ZERO,
		};

		assert_eq!(
			HashSet::from([
				StateMeta::Entering(_State::A),
				StateMeta::In(_State::A),
				StateMeta::Entering(_State::B),
				StateMeta::In(_State::B),
				StateMeta::Entering(_State::C),
				StateMeta::In(_State::C),
				StateMeta::Done,
			]),
			agent.updated_states(Duration::from_millis(10))
		);
	}

	#[test]
	fn get_all_states_with_all_zero_time() {
		let mut agent = _Agent {
			state_a: Duration::ZERO,
			state_b: Duration::ZERO,
			state_c: Duration::ZERO,
			elapsed: Duration::ZERO,
		};

		assert_eq!(
			HashSet::from([
				StateMeta::Entering(_State::A),
				StateMeta::In(_State::A),
				StateMeta::Entering(_State::B),
				StateMeta::In(_State::B),
				StateMeta::Entering(_State::C),
				StateMeta::In(_State::C),
				StateMeta::Done,
			]),
			agent.updated_states(Duration::from_millis(10))
		);
	}

	#[test]
	fn get_states_with_zero_duration_intermixed() {
		let mut agent = _Agent {
			state_a: Duration::from_secs(4),
			state_b: Duration::ZERO,
			state_c: Duration::from_secs(4),
			elapsed: Duration::ZERO,
		};

		assert_eq!(
			HashSet::from([
				StateMeta::Entering(_State::A),
				StateMeta::In(_State::A),
				StateMeta::Entering(_State::B),
				StateMeta::In(_State::B),
				StateMeta::Entering(_State::C),
				StateMeta::In(_State::C),
			]),
			agent.updated_states(Duration::from_secs(7))
		);
	}

	#[test]
	fn get_state_after_duration_of_a_zero_duration_state() {
		let mut agent = _Agent {
			state_a: Duration::from_secs(4),
			state_b: Duration::ZERO,
			state_c: Duration::from_secs(4),
			elapsed: Duration::from_secs(5),
		};

		assert_eq!(
			HashSet::from([StateMeta::In(_State::C),]),
			agent.updated_states(Duration::from_secs(1))
		);
	}

	#[test]
	fn can_handle_infinite_durations() {
		let mut agent = _Agent {
			state_a: Duration::MAX,
			state_b: Duration::from_secs(1),
			state_c: Duration::from_secs(1),
			elapsed: Duration::ZERO,
		};

		assert_eq!(
			HashSet::from([StateMeta::Entering(_State::A), StateMeta::In(_State::A)]),
			agent.updated_states(Duration::from_secs(1))
		);
	}

	#[test]
	fn get_all_progressively() {
		let mut agent = _Agent {
			state_a: Duration::from_secs(2),
			state_b: Duration::from_secs(2),
			state_c: Duration::from_secs(2),
			elapsed: Duration::ZERO,
		};

		let states = [
			agent.updated_states(Duration::from_secs(1)),
			agent.updated_states(Duration::from_secs(1)),
			agent.updated_states(Duration::from_secs(1)),
			agent.updated_states(Duration::from_secs(1)),
			agent.updated_states(Duration::from_secs(1)),
			agent.updated_states(Duration::from_secs(1)),
			agent.updated_states(Duration::from_secs(1)),
		];

		assert_eq!(
			[
				HashSet::from([StateMeta::Entering(_State::A), StateMeta::In(_State::A)]),
				HashSet::from([StateMeta::In(_State::A)]),
				HashSet::from([StateMeta::Entering(_State::B), StateMeta::In(_State::B)]),
				HashSet::from([StateMeta::In(_State::B)]),
				HashSet::from([StateMeta::Entering(_State::C), StateMeta::In(_State::C)]),
				HashSet::from([StateMeta::In(_State::C)]),
				HashSet::from([StateMeta::Done]),
			],
			states
		);
	}
}
