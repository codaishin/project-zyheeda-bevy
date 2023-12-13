pub mod game_state;

pub trait GetState<T> {
	fn get_state() -> Self;
}

#[cfg(test)]
pub mod test_tools {
	use super::*;

	pub fn get<S: GetState<T>, T>() -> S {
		S::get_state()
	}
}
