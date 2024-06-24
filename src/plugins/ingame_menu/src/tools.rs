pub(crate) mod menu_state;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PanelState {
	Empty,
	Filled,
}

#[cfg(test)]
pub(crate) mod test_tools {
	use bevy::{
		app::App,
		ecs::component::ComponentId,
		prelude::Component,
		render::view::{InheritedVisibility, ViewVisibility, Visibility},
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
	use std::any::type_name;

	type ComponentName = &'static str;

	pub(crate) trait ComponentNameAndId {
		fn name_and_id(app: &App) -> (ComponentName, Option<ComponentId>);
	}

	impl<TComponent: Component> ComponentNameAndId for TComponent {
		fn name_and_id(app: &App) -> (ComponentName, Option<ComponentId>) {
			(
				type_name::<TComponent>(),
				app.world.component_id::<TComponent>(),
			)
		}
	}

	pub(crate) trait BundleIds {
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

	pub(crate) fn get_ids<T: BundleIds>(app: &App) -> Vec<(ComponentName, Option<ComponentId>)> {
		T::bundle_ids(app)
	}

	macro_rules! assert_bundle {
		($bundle:ty, $app:expr, $entity:expr) => {
			let names_and_ids = crate::tools::test_tools::get_ids::<$bundle>($app);
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

			struct With<TComponent: Component, TAssert: Fn(&TComponent)>(
				TAssert,
				std::marker::PhantomData<TComponent>
			);

			impl <TComponent: Component, TAssert: Fn(&TComponent)> With<TComponent, TAssert> {
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

	pub(crate) use assert_bundle;
}
