pub mod utils {
	use bevy::{prelude::*, time::Time};
	use std::time::Duration;

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

	#[macro_export]
	macro_rules! assert_eq_approx {
		($left:expr, $right:expr, $tolerance:expr) => {
			match (&$left, &$right, &$tolerance) {
				(left_val, right_val, tolerance_val) => {
					assert!(
						$crate::test_tools::utils::approx_equal(left_val, right_val, tolerance_val),
						"approx equal failed:\n     left: {}\n    right: {}\ntolerance: {}\n",
						format!("\x1b[31m{:?}\x1b[0m", left_val),
						format!("\x1b[31m{:?}\x1b[0m", right_val),
						format!("\x1b[33m{:?}\x1b[0m", tolerance_val),
					);
				}
			}
		};
	}

	pub use assert_eq_approx;

	pub trait GetImmediateChildren
	where
		Self: Copy,
	{
		fn get_immediate_children(entity: &Entity, app: &App) -> Vec<Self>;
	}

	impl GetImmediateChildren for Entity {
		fn get_immediate_children(entity: &Entity, app: &App) -> Vec<Self> {
			match app.world.entity(*entity).get::<Children>() {
				None => vec![],
				Some(children) => children.iter().cloned().collect(),
			}
		}
	}

	pub trait GetImmediateChildComponents
	where
		Self: Component,
	{
		fn get_immediate_children<'a>(entity: &Entity, app: &'a App) -> Vec<&'a Self>;
	}

	impl<TComponent: Component> GetImmediateChildComponents for TComponent {
		fn get_immediate_children<'a>(entity: &Entity, app: &'a App) -> Vec<&'a Self> {
			Entity::get_immediate_children(entity, app)
				.iter()
				.filter_map(|entity| app.world.entity(*entity).get::<TComponent>())
				.collect()
		}
	}

	pub trait TickTime {
		fn tick_time(&mut self, delta: Duration);
	}

	impl TickTime for App {
		fn tick_time(&mut self, delta: Duration) {
			let mut time = self.world.resource_mut::<Time<Real>>();
			if time.last_update().is_none() {
				time.update();
			}
			let last_update = time.last_update().unwrap();
			time.update_with_instant(last_update + delta);
		}
	}
}
