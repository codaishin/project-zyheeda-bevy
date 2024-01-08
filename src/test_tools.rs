use bevy::prelude::*;

pub trait ApproxEqual<TTolerance> {
	fn approx_equal(&self, other: &Self, tolerance: &TTolerance) -> bool;
}

impl ApproxEqual<f32> for Vec3 {
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		self.abs_diff_eq(*other, *tolerance)
	}
}

impl<T: ApproxEqual<f32>> ApproxEqual<f32> for Option<T> {
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		match (self, other) {
			(None, None) => true,
			(Some(value_s), Some(value_o)) => value_s.approx_equal(value_o, tolerance),
			_ => false,
		}
	}
}

pub fn approx_equal<TEq: ApproxEqual<TT>, TT>(left: &TEq, right: &TEq, tolerance: &TT) -> bool {
	left.approx_equal(right, tolerance)
}

macro_rules! assert_eq_approx {
	($left:expr, $right:expr, $tolerance:expr) => {
		match (&$left, &$right, &$tolerance) {
			(left_val, right_val, tolerance_val) => {
				assert!(
					crate::test_tools::approx_equal(left_val, right_val, tolerance_val),
					"approx equal failed:\n     left: {}\n    right: {}\ntolerance: {}\n",
					format!("\x1b[31m{:?}\x1b[0m", left_val),
					format!("\x1b[31m{:?}\x1b[0m", right_val),
					format!("\x1b[33m{:?}\x1b[0m", tolerance_val),
				);
			}
		}
	};
}

pub(crate) use assert_eq_approx;
