pub trait Progress: internal::Progress {}

impl<T> Progress for T where T: internal::Progress {}

#[derive(Default, Debug, PartialEq)]
pub struct AssetLoadProgress;

#[derive(Default, Debug, PartialEq)]
pub struct DependencyResolveProgress;

mod internal {
	use super::*;

	pub trait Progress {}

	impl Progress for AssetLoadProgress {}
	impl Progress for DependencyResolveProgress {}
}
