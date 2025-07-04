use bevy::{
	ecs::schedule::{ExecutorKind, ScheduleLabel},
	prelude::*,
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

impl ApproxEqual<f32> for f32 {
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		(self - other).abs() <= *tolerance
	}
}

impl ApproxEqual<f32> for Vec3 {
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		self.abs_diff_eq(*other, *tolerance)
	}
}

impl ApproxEqual<f32> for Dir3 {
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		(*self).abs_diff_eq(**other, *tolerance)
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

impl<T, const N: usize> ApproxEqual<f32> for [T; N]
where
	T: ApproxEqual<f32>,
{
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		for (a, b) in self.iter().zip(other) {
			if !a.approx_equal(b, tolerance) {
				return false;
			}
		}

		true
	}
}

impl<T, TError> ApproxEqual<f32> for Result<T, TError>
where
	T: ApproxEqual<f32>,
	TError: PartialEq,
{
	fn approx_equal(&self, other: &Self, tolerance: &f32) -> bool {
		match (self, other) {
			(Err(a), Err(b)) => a == b,
			(Ok(a), Ok(b)) => a.approx_equal(b, tolerance),
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
					$crate::approx_equal(left_val, right_val, tolerance_val),
					"approx equal failed:\n    left: {}\n    right: {}\ntolerance: {}\n",
					format!("\x1b[31m{:?}\x1b[0m", left_val),
					format!("\x1b[31m{:?}\x1b[0m", right_val),
					format!("\x1b[33m{:?}\x1b[0m", tolerance_val),
				);
			}
		}
	};
}

#[macro_export]
macro_rules! get_children {
	($app:expr, $parent:expr) => {
		$app.world().iter_entities().filter(|entity| {
			entity
				.get::<bevy::prelude::ChildOf>()
				.map(|p| p.parent() == $parent)
				.unwrap_or(false)
		})
	};
}

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

pub trait EqualUnordered {
	type TItem;

	fn item_count(&self, item: &Self::TItem) -> usize;
	fn items(&self) -> Option<impl Iterator<Item = &Self::TItem>>;
}

impl<T> EqualUnordered for Vec<T>
where
	T: PartialEq,
{
	type TItem = T;

	fn item_count(&self, item: &Self::TItem) -> usize {
		self.iter().filter(|i| i == &item).count()
	}

	fn items(&self) -> Option<impl Iterator<Item = &Self::TItem>> {
		Some(self.iter())
	}
}

impl<T, const N: usize> EqualUnordered for [T; N]
where
	T: PartialEq,
{
	type TItem = T;

	fn item_count(&self, item: &Self::TItem) -> usize {
		self.iter().filter(|i| i == &item).count()
	}

	fn items(&self) -> Option<impl Iterator<Item = &Self::TItem>> {
		Some(self.iter())
	}
}

impl<T> EqualUnordered for Option<T>
where
	T: EqualUnordered,
{
	type TItem = T::TItem;

	fn item_count(&self, item: &Self::TItem) -> usize {
		match self {
			None => 0,
			Some(collection) => collection.item_count(item),
		}
	}

	fn items(&self) -> Option<impl Iterator<Item = &Self::TItem>> {
		match self {
			None => None,
			Some(collection) => collection.items(),
		}
	}
}

impl<T, TError> EqualUnordered for Result<T, TError>
where
	T: EqualUnordered,
	TError: PartialEq,
{
	type TItem = T::TItem;

	fn item_count(&self, item: &Self::TItem) -> usize {
		match self {
			Err(_) => 0,
			Ok(collection) => collection.item_count(item),
		}
	}

	fn items(&self) -> Option<impl Iterator<Item = &Self::TItem>> {
		match self {
			Err(_) => None,
			Ok(collection) => collection.items(),
		}
	}
}

pub fn equal_unordered<TEq: EqualUnordered>(left: &TEq, right: &TEq) -> bool {
	match (left.items(), right.items()) {
		(None, None) => true,
		(Some(mut left_items), Some(_)) => {
			left_items.all(|item| right.item_count(item) == left.item_count(item))
		}
		_ => false,
	}
}

#[macro_export]
macro_rules! assert_eq_unordered {
	($left:expr, $right:expr) => {
		match (&$left, &$right) {
			(left_val, right_val) => {
				assert!(
					$crate::equal_unordered(left_val, right_val),
					"unordered equal failed:\n  left: {}\n right: {}\n",
					format!("\x1b[31m{:?}\x1b[0m", left_val),
					format!("\x1b[31m{:?}\x1b[0m", right_val),
				);
			}
		}
	};
}

/// A simple wrapper around an expression to communicate intent.
///
/// It just executes the expression.
#[macro_export]
macro_rules! assert_no_panic {
	($expr:expr) => {
		$expr
	};
}

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
	new_handle_from(Uuid::new_v4())
}

pub fn new_handle_from<TAsset: Asset>(uuid: Uuid) -> Handle<TAsset> {
	Handle::Weak(AssetId::Uuid { uuid })
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

#[macro_export]
macro_rules! repeat_scope {
	($count:expr, $scope:expr $(,)?) => {{
		for _ in 0..$count {
			$scope
		}
	}};
}

pub trait NestedMocks<TMock> {
	fn with_mock(self, configure_mock_fn: impl FnMut(&mut TMock)) -> Self;
}

pub trait Mock {
	fn new_mock(configure: impl FnMut(&mut Self)) -> Self;
}

#[macro_export]
macro_rules! simple_init {
	($ident:ident) => {
		impl $crate::Mock for $ident {
			fn new_mock(mut configure: impl FnMut(&mut Self)) -> Self {
				let mut mock = Self::default();
				configure(&mut mock);
				mock
			}
		}
	};
}
