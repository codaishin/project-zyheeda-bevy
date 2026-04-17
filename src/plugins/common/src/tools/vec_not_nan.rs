use bevy::prelude::*;
use std::hash::Hash;
use zyheeda_core::prelude::*;

#[macro_export]
macro_rules! vec_not_nan {
	($($c:literal),+ $(,)?) => {{
		$crate::tools::vec_not_nan::VecNotNan([
			$(zyheeda_core::prelude::f32_not_nan!($c),)+
		])
	}};
}

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct VecNotNan<const N: usize>(pub [F32NotNan; N]);

impl<const N: usize> VecNotNan<N> {
	pub const ZERO: Self = Self([F32NotNan::ZERO; N]);
}

impl<const N: usize> Default for VecNotNan<N> {
	fn default() -> Self {
		Self([F32NotNan::ZERO; N])
	}
}

impl<const N: usize> TryFrom<[f32; N]> for VecNotNan<N> {
	type Error = IsNaN<N>;

	fn try_from(array: [f32; N]) -> Result<Self, Self::Error> {
		let mut vec = [F32NotNan::ZERO; N];

		for i in 0..N {
			let Ok(value) = F32NotNan::try_from_f32(array[i]) else {
				return Err(IsNaN(array));
			};

			vec[i] = value;
		}

		Ok(Self(vec))
	}
}

impl TryFrom<Vec3> for VecNotNan<3> {
	type Error = IsNaN<3>;

	fn try_from(vec: Vec3) -> Result<Self, Self::Error> {
		Self::try_from(vec.to_array())
	}
}

impl From<VecNotNan<3>> for Vec3 {
	fn from(VecNotNan([x, y, z]): VecNotNan<3>) -> Self {
		Vec3::new(*x, *y, *z)
	}
}

impl From<&'_ VecNotNan<3>> for Vec3 {
	fn from(vec: &VecNotNan<3>) -> Self {
		Vec3::from(*vec)
	}
}

impl From<VecNotNan<2>> for Vec2 {
	fn from(VecNotNan([x, y]): VecNotNan<2>) -> Self {
		Vec2::new(*x, *y)
	}
}

impl From<&'_ VecNotNan<2>> for Vec2 {
	fn from(vec: &VecNotNan<2>) -> Self {
		Vec2::from(*vec)
	}
}

#[derive(Debug)]
pub struct IsNaN<const N: usize>(#[allow(unused)] [f32; N]);

#[cfg(test)]
impl<const N: usize> PartialEq for IsNaN<N> {
	fn eq(&self, other: &Self) -> bool {
		for (a, b) in self.0.iter().zip(other.0.iter()) {
			if a == b || a.is_nan() && b.is_nan() {
				continue;
			}

			return false;
		}

		true
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;

	#[test]
	fn crate_node() {
		let node = VecNotNan::try_from([1., 2., 3.]);

		assert_eq!(
			Ok(const { VecNotNan([f32_not_nan!(1.), f32_not_nan!(2.), f32_not_nan!(3.)]) }),
			node
		);
	}

	#[test]
	fn crate_node_nan_fails() {
		let node = VecNotNan::try_from(Vec3::NAN);

		assert_eq!(Err(IsNaN([f32::NAN, f32::NAN, f32::NAN])), node);
	}
}
