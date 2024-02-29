pub trait GetState<T> {
	fn get_state() -> Self;
}

pub mod test_tools {
	use super::*;

	pub fn get<S: GetState<T>, T>() -> S {
		S::get_state()
	}
}
