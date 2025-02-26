use std::ops::{Add, AddAssign, Sub};

use bevy::math::Vec3;
use common::traits::handles_map_generation::NavCell;

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

	pub(crate) fn right_angle_len(&self) -> u32 {
		(self.x.abs() + self.y.abs()) as u32
	}

	pub(crate) fn is_straight(&self) -> bool {
		(self.x == 0 && self.y != 0) || (self.x != 0 && self.y == 0)
	}

	pub(crate) fn is_diagonal(&self) -> bool {
		self.x.abs() == self.y.abs()
	}

	pub(crate) fn eight_sided_direction_to(&self, target: &NavGridNode) -> Option<NavGridNode> {
		if self == target {
			return None;
		}

		let direction = *target - *self;

		if direction.x == 0 && direction.y != 0 {
			return Some(NavGridNode {
				x: 0,
				y: unit(direction.y),
			});
		};

		if direction.y == 0 && direction.x != 0 {
			return Some(NavGridNode {
				x: unit(direction.x),
				y: 0,
			});
		}

		if direction.x.abs() == direction.y.abs() {
			return Some(NavGridNode {
				x: unit(direction.x),
				y: unit(direction.y),
			});
		}

		None
	}
}

impl From<Vec3> for NavGridNode {
	fn from(Vec3 { x, z, .. }: Vec3) -> Self {
		Self {
			x: x as i32,
			y: z as i32,
		}
	}
}

impl From<NavGridNode> for Vec3 {
	fn from(NavGridNode { x, y }: NavGridNode) -> Self {
		Vec3 {
			x: x as f32,
			y: 0.,
			z: y as f32,
		}
	}
}

impl From<NavCell> for NavGridNode {
	fn from(cell: NavCell) -> Self {
		Self::from(&cell)
	}
}

impl From<&NavCell> for NavGridNode {
	fn from(NavCell { translation, .. }: &NavCell) -> Self {
		Self {
			x: translation.x as i32,
			y: translation.z as i32,
		}
	}
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

fn unit(value: i32) -> i32 {
	if value < 0 {
		-1
	} else {
		1
	}
}
