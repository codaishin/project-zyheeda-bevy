pub mod utils {
	use bevy::{
		app::App,
		ecs::schedule::{ExecutorKind, ScheduleLabel},
		prelude::{Component, *},
		time::Time,
	};
	use bevy_rapier3d::prelude::Velocity;
	use std::{
		any::{Any, TypeId},
		marker::PhantomData,
		time::Duration,
	};
	use uuid::Uuid;

	pub trait ApproxEqual<TTolerance> {
		fn approx_equal(&self, other: &Self, tolerance: &TTolerance) -> bool;
	}

	impl ApproxEqual<f32> for Vec3 {
		fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
			self.abs_diff_eq(*other, *tolerance)
		}
	}

	impl ApproxEqual<f32> for Quat {
		fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
			self.abs_diff_eq(*other, *tolerance)
		}
	}

	impl ApproxEqual<f32> for Transform {
		fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
			self.translation.approx_equal(&other.translation, tolerance)
				&& self.scale.approx_equal(&other.scale, tolerance)
				&& self.rotation.approx_equal(&other.rotation, tolerance)
		}
	}

	impl ApproxEqual<f32> for Velocity {
		fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
			self.linvel.approx_equal(&other.linvel, tolerance)
				&& self.angvel.approx_equal(&other.angvel, tolerance)
		}
	}

	impl<T> ApproxEqual<f32> for &T
	where
		T: ApproxEqual<f32>,
	{
		fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
			(*self).approx_equal(other, tolerance)
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
						"approx equal failed:\n    left: {}\n    right: {}\ntolerance: {}\n",
						format!("\x1b[31m{:?}\x1b[0m", left_val),
						format!("\x1b[31m{:?}\x1b[0m", right_val),
						format!("\x1b[33m{:?}\x1b[0m", tolerance_val),
					);
				}
			}
		};
	}

	pub use assert_eq_approx;

	#[macro_export]
	macro_rules! get_children {
		($app:expr, $parent:expr) => {
			$app.world().iter_entities().filter(|entity| {
				entity
					.get::<bevy::prelude::Parent>()
					.map(|p| p.get() == $parent)
					.unwrap_or(false)
			})
		};
	}

	pub use get_children;

	#[macro_export]
	macro_rules! assert_count {
		($count:literal, $iterator:expr) => {{
			let vec = $iterator.collect::<Vec<_>>();
			let vec_len = vec.len();
			let Ok(array) = <[_; $count]>::try_from(vec) else {
				panic!(
					"assert count failed:\n    expected: {}\n    actual: {}\n",
					format!("\x1b[32m{:?}\x1b[0m", $count),
					format!("\x1b[31m{:?}\x1b[0m", vec_len),
				);
			};

			array
		}};
	}

	pub use assert_count;

	enum Iter<T> {
		None,
		Some(T),
	}

	impl<T> Iterator for Iter<T>
	where
		T: Iterator,
	{
		type Item = T::Item;

		fn next(&mut self) -> Option<Self::Item> {
			match self {
				Iter::None => None,
				Iter::Some(iter) => iter.next(),
			}
		}
	}

	pub trait EqualUnordered {
		type TItem;

		fn buffer_eq(&self, rhs: &Self) -> bool;
		fn buffer_contains(&self, item: &Self::TItem) -> bool;
		fn items(&self) -> impl Iterator<Item = &Self::TItem>;
	}

	impl<T> EqualUnordered for Vec<T>
	where
		T: PartialEq,
	{
		type TItem = T;

		fn buffer_eq(&self, rhs: &Self) -> bool {
			self.len() == rhs.len()
		}

		fn buffer_contains(&self, item: &Self::TItem) -> bool {
			self.contains(item)
		}

		fn items(&self) -> impl Iterator<Item = &Self::TItem> {
			self.iter()
		}
	}

	impl<T, const N: usize> EqualUnordered for [T; N]
	where
		T: PartialEq,
	{
		type TItem = T;

		fn buffer_eq(&self, _: &Self) -> bool {
			true
		}

		fn buffer_contains(&self, item: &Self::TItem) -> bool {
			self.contains(item)
		}

		fn items(&self) -> impl Iterator<Item = &Self::TItem> {
			self.iter()
		}
	}

	impl<T> EqualUnordered for Option<T>
	where
		T: EqualUnordered,
	{
		type TItem = T::TItem;

		fn buffer_eq(&self, rhs: &Self) -> bool {
			match (self, rhs) {
				(None, None) => true,
				(Some(a), Some(b)) => a.buffer_eq(b),
				_ => false,
			}
		}

		fn buffer_contains(&self, item: &Self::TItem) -> bool {
			match self {
				None => false,
				Some(collection) => collection.buffer_contains(item),
			}
		}

		fn items(&self) -> impl Iterator<Item = &Self::TItem> {
			match self {
				None => Iter::None,
				Some(collection) => Iter::Some(collection.items()),
			}
		}
	}

	impl<T, TError> EqualUnordered for Result<T, TError>
	where
		T: EqualUnordered,
		TError: PartialEq,
	{
		type TItem = T::TItem;

		fn buffer_eq(&self, rhs: &Self) -> bool {
			match (self, rhs) {
				(Err(a), Err(b)) => a == b,
				(Ok(a), Ok(b)) => a.buffer_eq(b),
				_ => false,
			}
		}

		fn buffer_contains(&self, item: &Self::TItem) -> bool {
			match self {
				Err(_) => false,
				Ok(collection) => collection.buffer_contains(item),
			}
		}

		fn items(&self) -> impl Iterator<Item = &Self::TItem> {
			match self {
				Err(_) => Iter::None,
				Ok(collection) => Iter::Some(collection.items()),
			}
		}
	}

	pub fn equal_unordered<TEq: EqualUnordered>(left: &TEq, right: &TEq) -> bool {
		if !left.buffer_eq(right) {
			return false;
		}

		left.items().all(|item| right.buffer_contains(item))
	}

	#[macro_export]
	macro_rules! assert_eq_unordered {
		($left:expr, $right:expr) => {
			match (&$left, &$right) {
				(left_val, right_val) => {
					assert!(
						$crate::test_tools::utils::equal_unordered(left_val, right_val),
						"unordered equal failed:\n  left: {}\n right: {}\n",
						format!("\x1b[31m{:?}\x1b[0m", left_val),
						format!("\x1b[31m{:?}\x1b[0m", right_val),
					);
				}
			}
		};
	}

	pub use assert_eq_unordered;

	pub trait TickTime {
		fn tick_time(&mut self, delta: Duration);
	}

	impl TickTime for App {
		fn tick_time(&mut self, delta: Duration) {
			let mut time = self.world_mut().resource_mut::<Time<Real>>();
			if time.last_update().is_none() {
				time.update();
			}
			let last_update = time.last_update().unwrap();
			time.update_with_instant(last_update + delta);
		}
	}

	pub trait SingleThreadedApp {
		fn single_threaded(self, label: impl ScheduleLabel) -> Self;
	}

	impl SingleThreadedApp for App {
		fn single_threaded(mut self, label: impl ScheduleLabel) -> Self {
			self.edit_schedule(label, |schedule| {
				schedule.set_executor_kind(ExecutorKind::SingleThreaded);
			});

			self
		}
	}

	#[derive(Debug, PartialEq)]
	pub struct DownCastError(TypeId);

	pub trait TryCast {
		fn try_cast<TTarget: 'static>(&self) -> Result<&TTarget, DownCastError>;
	}

	impl<TAny: 'static> TryCast for TAny {
		/// Use only for unit/integration tests
		fn try_cast<TTarget: 'static>(&self) -> Result<&TTarget, DownCastError> {
			match (self as &dyn Any).downcast_ref::<TTarget>() {
				Some(down_casted) => Ok(down_casted),
				None => Err(DownCastError(self.type_id())),
			}
		}
	}

	pub fn new_handle<TAsset: Asset>() -> Handle<TAsset> {
		Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		})
	}

	#[derive(Component, Debug, PartialEq)]
	pub struct Changed<T: Component> {
		pub changed: bool,
		phantom_date: PhantomData<T>,
	}

	impl<T: Component> Changed<T> {
		pub fn new(changed: bool) -> Self {
			Self {
				changed,
				phantom_date: PhantomData,
			}
		}

		pub fn detect(mut query: Query<(Ref<T>, &mut Changed<T>)>) {
			for (component, mut changed) in &mut query {
				changed.changed = component.is_changed();
			}
		}
	}

	#[macro_export]
	macro_rules! is_changed_resource {
		($ty:ty, $result:expr) => {{
			let result = $result.clone();
			move |resource: Res<$ty>| {
				*result.lock().unwrap() = resource.is_changed();
			}
		}};
	}

	pub use is_changed_resource;
}
