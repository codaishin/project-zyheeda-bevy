use bevy::ui::{UiRect, Val};
use common::{tools::Index, traits::accessors::get::Property};
use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PanelState {
	Empty,
	Filled,
}

impl Property for PanelState {
	type TValue<'a> = Self;
}

pub enum Layout {
	LastColumn(Index<u16>),
	LastRow(Index<u16>),
}

impl Layout {
	pub const SINGLE_COLUMN: Layout = Layout::LastColumn(Index(0));
	pub const SINGLE_ROW: Layout = Layout::LastRow(Index(0));
}

impl Default for Layout {
	fn default() -> Self {
		Self::SINGLE_COLUMN
	}
}

#[derive(Default, Debug, PartialEq)]
pub(crate) struct Pixel(pub f32);

impl From<Pixel> for Val {
	fn from(value: Pixel) -> Self {
		Val::Px(value.0)
	}
}

impl From<Pixel> for UiRect {
	fn from(value: Pixel) -> Self {
		UiRect::all(Val::from(value))
	}
}

impl Sub for Pixel {
	type Output = Pixel;

	fn sub(self, rhs: Self) -> Self::Output {
		Pixel(self.0 - rhs.0)
	}
}

impl Add for Pixel {
	type Output = Pixel;

	fn add(self, rhs: Self) -> Self::Output {
		Pixel(self.0 + rhs.0)
	}
}

impl Div<f32> for Pixel {
	type Output = Pixel;

	fn div(self, rhs: f32) -> Self::Output {
		Pixel(self.0 / rhs)
	}
}

impl Mul<f32> for Pixel {
	type Output = Pixel;

	fn mul(self, rhs: f32) -> Self::Output {
		Pixel(self.0 * rhs)
	}
}

#[derive(Default)]
pub(crate) struct Dimensions {
	pub(crate) width: Pixel,
	pub(crate) height: Pixel,
	pub(crate) border: Pixel,
}

impl Dimensions {
	pub(crate) fn height_inner(&self) -> Pixel {
		Pixel(self.height.0 - self.border.0)
	}

	pub(crate) fn height_outer(&self) -> Pixel {
		Pixel(self.height.0 + self.border.0)
	}

	pub(crate) fn width_inner(&self) -> Pixel {
		Pixel(self.width.0 - self.border.0)
	}

	pub(crate) fn width_outer(&self) -> Pixel {
		Pixel(self.width.0 + self.border.0)
	}

	pub(crate) fn minimum_inner(&self) -> Pixel {
		Pixel(-self.border.0)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn pixel_42_sub_10() {
		let pixel = Pixel(42.);

		assert_eq!(Pixel(10.), pixel - Pixel(32.))
	}

	#[test]
	fn pixel_11_sub_6() {
		let pixel = Pixel(11.);

		assert_eq!(Pixel(5.), pixel - Pixel(6.))
	}

	#[test]
	fn pixel_42_add_10() {
		let pixel = Pixel(42.);

		assert_eq!(Pixel(74.), pixel + Pixel(32.))
	}

	#[test]
	fn pixel_11_add_6() {
		let pixel = Pixel(11.);

		assert_eq!(Pixel(17.), pixel + Pixel(6.))
	}

	#[test]
	fn pixel_42_div_10() {
		let pixel = Pixel(42.);

		assert_eq!(Pixel(42. / 32.), pixel / 32.)
	}

	#[test]
	fn pixel_11_div_6() {
		let pixel = Pixel(11.);

		assert_eq!(Pixel(11. / 6.), pixel / 6.)
	}

	#[test]
	fn pixel_42_mul_10() {
		let pixel = Pixel(42.);

		assert_eq!(Pixel(42. * 32.), pixel * 32.)
	}

	#[test]
	fn pixel_11_mul_6() {
		let pixel = Pixel(11.);

		assert_eq!(Pixel(11. * 6.), pixel * 6.)
	}
}
