pub mod utils {
	use bevy::{
		app::App,
		ecs::{
			component::ComponentId,
			schedule::{ExecutorKind, ScheduleLabel},
		},
		prelude::{Component, *},
		render::view::{InheritedVisibility, ViewVisibility, Visibility},
		time::Time,
		transform::components::{GlobalTransform, Transform},
		ui::{
			node_bundles::NodeBundle,
			BackgroundColor,
			BorderColor,
			FocusPolicy,
			Node,
			Style,
			ZIndex,
		},
	};
	use std::{
		any::{type_name, Any, TypeId},
		time::Duration,
	};

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

	type ComponentName = &'static str;

	pub trait ComponentNameAndId {
		fn name_and_id(app: &App) -> (ComponentName, Option<ComponentId>);
	}

	impl<TComponent: Component> ComponentNameAndId for TComponent {
		fn name_and_id(app: &App) -> (ComponentName, Option<ComponentId>) {
			(
				type_name::<TComponent>(),
				app.world().component_id::<TComponent>(),
			)
		}
	}

	pub trait BundleIds {
		fn bundle_ids(app: &App) -> Vec<(ComponentName, Option<ComponentId>)>;
	}

	impl BundleIds for NodeBundle {
		fn bundle_ids(app: &App) -> Vec<(ComponentName, Option<ComponentId>)> {
			vec![
				Node::name_and_id(app),
				Style::name_and_id(app),
				BackgroundColor::name_and_id(app),
				BorderColor::name_and_id(app),
				FocusPolicy::name_and_id(app),
				Transform::name_and_id(app),
				GlobalTransform::name_and_id(app),
				Visibility::name_and_id(app),
				InheritedVisibility::name_and_id(app),
				ViewVisibility::name_and_id(app),
				ZIndex::name_and_id(app),
			]
		}
	}

	impl BundleIds for SpatialBundle {
		fn bundle_ids(app: &App) -> Vec<(ComponentName, Option<ComponentId>)> {
			vec![
				Visibility::name_and_id(app),
				InheritedVisibility::name_and_id(app),
				ViewVisibility::name_and_id(app),
				Transform::name_and_id(app),
				GlobalTransform::name_and_id(app),
			]
		}
	}

	pub fn get_ids<T: BundleIds>(app: &App) -> Vec<(ComponentName, Option<ComponentId>)> {
		T::bundle_ids(app)
	}

	#[macro_export]
	macro_rules! assert_bundle {
		($bundle:ty, $app:expr, $entity:expr) => {
			let names_and_ids = $crate::test_tools::utils::get_ids::<$bundle>($app);
			let missing = names_and_ids
				.iter()
				.filter(|(_, id)| id.is_none() || !$entity.contains_id(id.unwrap()))
				.map(|(name, ..)| name)
				.collect::<Vec<_>>();

			if !missing.is_empty() {
				panic!(
					"Entity {:?}: Bundle <{}> incomplete with missing: {:#?}",
					$entity.id(),
					std::any::type_name::<$bundle>(),
					missing
				);
			}
		};

		($bundle:ty, $app:expr, $entity:expr, $assert:expr) => {{
			assert_bundle!($bundle,$app,  $entity);

			struct With<TComponent: bevy::ecs::component::Component, TAssert: Fn(&TComponent)>(
				TAssert,
				std::marker::PhantomData<TComponent>
			);

			impl <TComponent: bevy::ecs::component::Component, TAssert: Fn(&TComponent)> With<TComponent, TAssert> {
				fn assert(assert_fn: TAssert) -> Self {
					Self(assert_fn, std::marker::PhantomData)
				}

				fn execute(&self, entity: bevy::ecs::world::EntityRef) {
					let component = entity.get::<TComponent>().unwrap_or_else(|| panic!(
						"Entity {:?} does not contain a component of type <{}>",
						entity.id(),
						std::any::type_name::<TComponent>(),
					));
					(self.0)(component);
				}
			}

			$assert.execute($entity);
		}};

		($bundle:ty, $app:expr,$entity:expr, $assert:expr, $($rest:expr),+) => {{
			assert_bundle!($bundle, $app, $entity, $assert);
			assert_bundle!($bundle, $app, $entity, $($rest),+);
		}};
	}

	pub use assert_bundle;
}
