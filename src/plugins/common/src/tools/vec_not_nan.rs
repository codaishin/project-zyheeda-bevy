use bevy::prelude::*;
use std::{cmp::Ordering, hash::Hash, ops::Deref};
use zyheeda_core::prelude::*;

#[macro_export]
macro_rules! vec_not_nan {
	($($c:literal),+ $(,)?) => {{
		$crate::tools::vec_not_nan::VecNotNan([
			$(zyheeda_core::prelude::f32_not_nan!($c),)+
		])
	}};
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct VecNotNan<const N: usize>(pub [F32NotNan; N]);

impl<const N: usize> VecNotNan<N> {
	pub const ZERO: Self = Self([F32NotNan::ZERO; N]);

	pub const fn try_from_array(array: [f32; N]) -> Result<Self, IsNaN<N>> {
		let mut vec = [F32NotNan::ZERO; N];
		let mut i = 0;

		while i < N {
			let Ok(value) = F32NotNan::try_from_f32(array[i]) else {
				return Err(IsNaN(array));
			};

			vec[i] = value;

			i += 1;
		}

		Ok(Self(vec))
	}
}

impl<const N: usize> Default for VecNotNan<N> {
	fn default() -> Self {
		Self([F32NotNan::ZERO; N])
	}
}

impl<const N: usize> Eq for VecNotNan<N> {}

impl<const N: usize> Hash for VecNotNan<N> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		for v in self.0 {
			let bits = match v {
				F32NotNan::ZERO => 0,
				v => v.to_bits(),
			};

			bits.hash(state);
		}
	}
}

impl<const N: usize> Ord for VecNotNan<N> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.0
			.iter()
			.zip(other.0)
			.fold(Ordering::Equal, |acc, (a, b)| acc.then(a.total_cmp(&b)))
	}
}

impl<const N: usize> PartialOrd for VecNotNan<N> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl TryFrom<Vec3> for VecNotNan<3> {
	type Error = IsNaN<3>;

	fn try_from(vec: Vec3) -> Result<Self, Self::Error> {
		Self::try_from_array(vec.to_array())
	}
}

impl From<VecNotNan<3>> for Vec3 {
	fn from(VecNotNan(vec): VecNotNan<3>) -> Self {
		Vec3::from_array(vec.map(|v| *v.deref()))
	}
}

impl From<&'_ VecNotNan<3>> for Vec3 {
	fn from(vec: &VecNotNan<3>) -> Self {
		Vec3::from(*vec)
	}
}

impl From<VecNotNan<2>> for Vec2 {
	fn from(VecNotNan(vec): VecNotNan<2>) -> Self {
		Vec2::from_array(vec.map(|v| *v.deref()))
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
	use std::hash::{DefaultHasher, Hasher};
	use test_case::test_case;

	fn hash<const N: usize>(node: VecNotNan<N>) -> u64 {
		let mut hasher = DefaultHasher::new();
		node.hash(&mut hasher);
		hasher.finish()
	}

	#[test]
	fn crate_node() {
		let node = VecNotNan::try_from(Vec3::new(1., 2., 3.));

		assert_eq!(
			Ok(const { VecNotNan([f32_not_nan!(1.), f32_not_nan!(2.), f32_not_nan!(3.),]) }),
			node
		);
	}

	#[test]
	fn crate_node_nan_fails() {
		let node = VecNotNan::try_from(Vec3::NAN);

		assert_eq!(Err(IsNaN([f32::NAN, f32::NAN, f32::NAN])), node);
	}

	#[test_case(Vec3::new(1., 2., 3.), Vec3::new(-1., 2., 3.); "x differs")]
	#[test_case(Vec3::new(1., 2., 3.), Vec3::new(1., -2., 3.); "y differs")]
	#[test_case(Vec3::new(1., 2., 3.), Vec3::new(1., 2., -3.); "z differs")]
	fn hashes_differ(a: Vec3, b: Vec3) -> Result<(), IsNaN<3>> {
		let a = VecNotNan::try_from(a)?;
		let b = VecNotNan::try_from(b)?;

		assert_ne!(hash(a), hash(b));
		Ok(())
	}

	#[test]
	fn hashes_match() -> Result<(), IsNaN<3>> {
		let a = VecNotNan::try_from(Vec3::new(1., 2., 3.))?;
		let b = VecNotNan::try_from(Vec3::new(1., 2., 3.))?;

		assert_eq!(hash(a), hash(b));
		Ok(())
	}

	#[test_case(Vec3::new(-0., 2., 3.); "x zero")]
	#[test_case(Vec3::new(1., -0., 3.); "y zero")]
	#[test_case(Vec3::new(1., 2., -0.); "z zero")]
	fn hashes_match_with_zero(v: Vec3) -> Result<(), IsNaN<3>> {
		let a = VecNotNan::try_from(v)?;
		let b = VecNotNan::try_from(v.abs())?;

		assert_eq!(hash(a), hash(b));
		Ok(())
	}
}
