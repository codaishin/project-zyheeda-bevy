use std::ops::{Add, AddAssign, Sub};

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Clone, Copy)]
pub struct NavGridNode {
	pub(crate) x: i32,
	pub(crate) y: i32,
}

impl NavGridNode {
	pub(crate) const MAX: NavGridNode = NavGridNode {
		x: i32::MAX,
		y: i32::MAX,
	};
	pub(crate) const MIN: NavGridNode = NavGridNode {
		x: i32::MIN,
		y: i32::MIN,
	};
}

impl Add for NavGridNode {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x + rhs.x,
			y: self.y + rhs.y,
		}
	}
}

impl AddAssign for NavGridNode {
	fn add_assign(&mut self, rhs: Self) {
		*self = *self + rhs;
	}
}

impl Sub for NavGridNode {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self {
			x: self.x - rhs.x,
			y: self.y - rhs.y,
		}
	}
}
