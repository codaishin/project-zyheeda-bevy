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
			match app.world().entity(*entity).get::<Children>() {
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
				.filter_map(|entity| app.world().entity(*entity).get::<TComponent>())
				.collect()
		}
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
