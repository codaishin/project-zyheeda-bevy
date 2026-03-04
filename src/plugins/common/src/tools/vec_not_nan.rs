use bevy::prelude::*;
use std::{cmp::Ordering, hash::Hash, ops::Deref};

#[macro_export]
macro_rules! vec3_not_nan {
	($x:literal, $y:literal, $z:literal $(,)?) => {{
		const VEC: Vec3NotNan = match Vec3NotNan::try_from_xyz($x, $y, $z) {
			Ok(vec) => vec,
			Err(_) => panic!("Vector is `NaN`"),
		};

		VEC
	}};
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Vec3NotNan(Vec3);

impl Vec3NotNan {
	pub const ZERO: Self = vec3_not_nan!(0., 0., 0.);

	pub const fn try_from_xyz(x: f32, y: f32, z: f32) -> Result<Self, IsNaN> {
		let vec = Vec3 { x, y, z };

		if x.is_nan() || y.is_nan() || z.is_nan() {
			return Err(IsNaN(vec));
		}

		Ok(Self(vec))
	}
}

impl Eq for Vec3NotNan {}

impl Hash for Vec3NotNan {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		for v in self.0.to_array() {
			let bits = match v {
				0.0 => 0,
				v => v.to_bits(),
			};

			bits.hash(state);
		}
	}
}

impl Ord for Vec3NotNan {
	fn cmp(&self, other: &Self) -> Ordering {
		self.0
			.x
			.total_cmp(&other.0.x)
			.then(self.0.y.total_cmp(&other.0.y))
			.then(self.0.z.total_cmp(&other.0.z))
	}
}

impl PartialOrd for Vec3NotNan {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl TryFrom<Vec3> for Vec3NotNan {
	type Error = IsNaN;

	fn try_from(Vec3 { x, y, z }: Vec3) -> Result<Self, Self::Error> {
		Self::try_from_xyz(x, y, z)
	}
}

impl From<Vec3NotNan> for Vec3 {
	fn from(Vec3NotNan(vec): Vec3NotNan) -> Self {
		vec
	}
}

impl Deref for Vec3NotNan {
	type Target = Vec3;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Debug)]
pub struct IsNaN(#[allow(unused)] Vec3);

#[cfg(test)]
impl PartialEq for IsNaN {
	fn eq(&self, other: &Self) -> bool {
		for (a, b) in self.0.to_array().into_iter().zip(other.0.to_array()) {
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
	use super::*;
	use std::hash::{DefaultHasher, Hasher};
	use test_case::test_case;

	fn hash(node: Vec3NotNan) -> u64 {
		let mut hasher = DefaultHasher::new();
		node.hash(&mut hasher);
		hasher.finish()
	}

	#[test]
	fn crate_node() {
		let node = Vec3NotNan::try_from(Vec3::new(1., 2., 3.));

		assert_eq!(Ok(Vec3::new(1., 2., 3.)), node.map(|n| *n));
	}

	#[test]
	fn crate_node_nan_fails() {
		let node = Vec3NotNan::try_from(Vec3::NAN);

		assert_eq!(Err(IsNaN(Vec3::NAN)), node.map(|n| *n));
	}

	#[test_case(Vec3::new(1., 2., 3.), Vec3::new(-1., 2., 3.); "x differs")]
	#[test_case(Vec3::new(1., 2., 3.), Vec3::new(1., -2., 3.); "y differs")]
	#[test_case(Vec3::new(1., 2., 3.), Vec3::new(1., 2., -3.); "z differs")]
	fn hashes_differ(a: Vec3, b: Vec3) -> Result<(), IsNaN> {
		let a = Vec3NotNan::try_from(a)?;
		let b = Vec3NotNan::try_from(b)?;

		assert_ne!(hash(a), hash(b));
		Ok(())
	}

	#[test]
	fn hashes_match() -> Result<(), IsNaN> {
		let a = Vec3NotNan::try_from(Vec3::new(1., 2., 3.))?;
		let b = Vec3NotNan::try_from(Vec3::new(1., 2., 3.))?;

		assert_eq!(hash(a), hash(b));
		Ok(())
	}

	#[test_case(Vec3::new(-0., 2., 3.); "x zero")]
	#[test_case(Vec3::new(1., -0., 3.); "y zero")]
	#[test_case(Vec3::new(1., 2., -0.); "z zero")]
	fn hashes_match_with_zero(a: Vec3) -> Result<(), IsNaN> {
		let a = Vec3NotNan::try_from(a)?;
		let b = Vec3NotNan::try_from(a.abs())?;

		assert_eq!(hash(a), hash(b));
		Ok(())
	}
}
