pub trait Progress: internal::Progress {}

impl<T> Progress for T where T: internal::Progress {}

#[derive(Default, Debug, PartialEq)]
pub struct AssetsProgress;

#[derive(Default, Debug, PartialEq)]
pub struct DependenciesProgress;

mod internal {
	use super::*;

	pub trait Progress {}

	impl Progress for AssetsProgress {}
	impl Progress for DependenciesProgress {}
}
