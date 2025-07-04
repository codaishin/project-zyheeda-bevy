use super::RegisterDerivedComponent;
use crate::traits::{
	register_derived_component::DerivableComponentFrom,
	thread_safe::ThreadSafe,
	try_insert_on::TryInsertOn,
};
use bevy::prelude::*;
use std::marker::PhantomData;

impl RegisterDerivedComponent for App {
	fn register_derived_component<TComponent, TDerived>(&mut self) -> &mut Self
	where
		TComponent: Component,
		for<'a> TDerived: DerivableComponentFrom<TComponent>,
	{
		if Observing::<TComponent, TDerived>::already_in(self) {
			return self;
		}

		match TDerived::INSERT {
			super::InsertDerivedComponent::IfNew => {
				self.add_observer(insert_if_new::<TComponent, TDerived>)
			}
			super::InsertDerivedComponent::Always => {
				self.add_observer(insert_always::<TComponent, TDerived>)
			}
		};

		self.init_resource::<Observing<TComponent, TDerived>>()
	}
}

fn insert_if_new<TComponent, TRequired>(
	trigger: Trigger<OnInsert, TComponent>,
	mut commands: Commands,
	components: Query<(&TComponent, Option<&TRequired>)>,
) where
	TComponent: Component,
	for<'a> TRequired: Component + From<&'a TComponent>,
{
	let entity = trigger.target();
	let Ok((component, None)) = components.get(entity) else {
		return;
	};
	commands.try_insert_on(entity, TRequired::from(component));
}

fn insert_always<TComponent, TRequired>(
	trigger: Trigger<OnInsert, TComponent>,
	mut commands: Commands,
	components: Query<&TComponent>,
) where
	TComponent: Component,
	for<'a> TRequired: Component + From<&'a TComponent>,
{
	let entity = trigger.target();
	let Ok(component) = components.get(entity) else {
		return;
	};
	commands.try_insert_on(entity, TRequired::from(component));
}

#[derive(Resource)]
struct Observing<TComponent, TRequired>(PhantomData<(TComponent, TRequired)>);

impl<TComponent, TRequired> Observing<TComponent, TRequired>
where
	TComponent: ThreadSafe,
	TRequired: ThreadSafe,
{
	fn already_in(app: &App) -> bool {
		app.world().get_resource::<Self>().is_some()
	}
}

impl<TComponent, TRequired> Default for Observing<TComponent, TRequired> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::register_derived_component::{
		DerivableComponentFrom,
		InsertDerivedComponent,
	};
	use testing::{SingleThreadedApp, assert_count};

	#[derive(Component)]
	struct _Component;

	#[derive(Component, Debug, PartialEq)]
	enum _NewlyDerived {
		A,
		B,
	}

	impl From<&_Component> for _NewlyDerived {
		fn from(_: &_Component) -> Self {
			_NewlyDerived::A
		}
	}

	impl DerivableComponentFrom<_Component> for _NewlyDerived {
		const INSERT: InsertDerivedComponent = InsertDerivedComponent::IfNew;
	}

	#[derive(Component, Debug, PartialEq)]
	enum _AlwaysDerived {
		A,
		B,
	}

	impl From<&_Component> for _AlwaysDerived {
		fn from(_: &_Component) -> Self {
			_AlwaysDerived::A
		}
	}

	impl DerivableComponentFrom<_Component> for _AlwaysDerived {
		const INSERT: InsertDerivedComponent = InsertDerivedComponent::Always;
	}

	fn setup<TRequired>() -> App
	where
		TRequired: DerivableComponentFrom<_Component>,
	{
		let mut app = App::new().single_threaded(Update);

		app.register_derived_component::<_Component, TRequired>();

		app
	}

	mod insert_if_new {
		use super::*;

		#[test]
		fn insert_derived() {
			let mut app = setup::<_NewlyDerived>();

			let entity = app.world_mut().spawn(_Component);

			assert_eq!(Some(&_NewlyDerived::A), entity.get::<_NewlyDerived>());
		}

		#[test]
		fn do_not_override_when_derived_already_present() {
			let mut app = setup::<_NewlyDerived>();

			let entity = app.world_mut().spawn((_Component, _NewlyDerived::B));

			assert_eq!(Some(&_NewlyDerived::B), entity.get::<_NewlyDerived>());
		}

		#[test]
		fn insert_again_when_reinserted() {
			let mut app = setup::<_NewlyDerived>();

			let mut entity = app.world_mut().spawn(_Component);
			entity.remove::<_NewlyDerived>();
			entity.insert(_Component);

			assert_eq!(Some(&_NewlyDerived::A), entity.get::<_NewlyDerived>());
		}

		#[test]
		fn prevent_multiple_observers() {
			let mut app = setup::<_NewlyDerived>();

			app.register_derived_component::<_Component, _NewlyDerived>();

			assert_count!(1, app.world().iter_entities());
		}
	}

	mod insert_always {
		use super::*;

		#[test]
		fn insert_derived() {
			let mut app = setup::<_AlwaysDerived>();

			let entity = app.world_mut().spawn(_Component);

			assert_eq!(Some(&_AlwaysDerived::A), entity.get::<_AlwaysDerived>());
		}

		#[test]
		fn override_when_derived_already_present() {
			let mut app = setup::<_AlwaysDerived>();

			let entity = app.world_mut().spawn((_Component, _AlwaysDerived::B));

			assert_eq!(Some(&_AlwaysDerived::A), entity.get::<_AlwaysDerived>());
		}

		#[test]
		fn insert_again_when_reinserted() {
			let mut app = setup::<_AlwaysDerived>();

			let mut entity = app.world_mut().spawn(_Component);
			entity.remove::<_AlwaysDerived>();
			entity.insert(_Component);

			assert_eq!(Some(&_AlwaysDerived::A), entity.get::<_AlwaysDerived>());
		}

		#[test]
		fn prevent_multiple_observers() {
			let mut app = setup::<_AlwaysDerived>();

			app.register_derived_component::<_Component, _AlwaysDerived>();

			assert_count!(1, app.world().iter_entities());
		}
	}
}
