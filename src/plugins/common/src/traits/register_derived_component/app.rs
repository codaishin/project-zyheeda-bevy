use super::RegisterDerivedComponent;
use crate::{
	traits::{
		accessors::get::TryApplyOn,
		register_derived_component::DerivableFrom,
		thread_safe::ThreadSafe,
	},
	zyheeda_commands::ZyheedaCommands,
};
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use std::marker::PhantomData;

impl RegisterDerivedComponent for App {
	fn register_derived_component<TComponent, TDerived>(&mut self) -> &mut Self
	where
		TComponent: Component,
		for<'w, 's> TDerived: DerivableFrom<'w, 's, TComponent>,
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

fn insert_if_new<TComponent, TDerived>(
	trigger: Trigger<OnInsert, TComponent>,
	mut commands: ZyheedaCommands,
	components: Query<&TComponent>,
	param: StaticSystemParam<<TDerived as DerivableFrom<'_, '_, TComponent>>::TParam>,
) where
	TComponent: Component,
	for<'w, 's> TDerived: DerivableFrom<'w, 's, TComponent>,
{
	let entity = trigger.target();
	let Ok(component) = components.get(entity) else {
		return;
	};
	let Some(derived) = TDerived::derive_from(entity, component, &param) else {
		return;
	};

	commands.try_apply_on(&entity, |mut e| {
		e.try_insert_if_new(derived);
	});
}

fn insert_always<TComponent, TDerived>(
	trigger: Trigger<OnInsert, TComponent>,
	mut commands: ZyheedaCommands,
	components: Query<&TComponent>,
	param: StaticSystemParam<<TDerived as DerivableFrom<'_, '_, TComponent>>::TParam>,
) where
	TComponent: Component,
	for<'w, 's> TDerived: DerivableFrom<'w, 's, TComponent>,
{
	let entity = trigger.target();
	let Ok(component) = components.get(entity) else {
		return;
	};
	let Some(derived) = TDerived::derive_from(entity, component, &param) else {
		return;
	};

	commands.try_apply_on(&entity, |mut e| {
		e.try_insert(derived);
	});
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
	use crate::traits::register_derived_component::{DerivableFrom, InsertDerivedComponent};
	use testing::{SingleThreadedApp, assert_count};

	#[derive(Component)]
	struct _Component;

	#[derive(Component, Debug, PartialEq)]
	enum _NewlyDerived {
		A(Entity),
		B,
	}

	impl<'w, 's> DerivableFrom<'w, 's, _Component> for _NewlyDerived {
		const INSERT: InsertDerivedComponent = InsertDerivedComponent::IfNew;

		type TParam = ();

		fn derive_from(entity: Entity, _: &_Component, _: &()) -> Option<Self> {
			Some(_NewlyDerived::A(entity))
		}
	}

	#[derive(Component, Debug, PartialEq)]
	enum _AlwaysDerived {
		A(Entity),
		B,
	}

	impl<'w, 's> DerivableFrom<'w, 's, _Component> for _AlwaysDerived {
		const INSERT: InsertDerivedComponent = InsertDerivedComponent::Always;

		type TParam = ();

		fn derive_from(entity: Entity, _: &_Component, _: &()) -> Option<Self> {
			Some(_AlwaysDerived::A(entity))
		}
	}

	fn setup<TRequired>() -> App
	where
		TRequired: for<'w, 's> DerivableFrom<'w, 's, _Component>,
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

			assert_eq!(
				Some(&_NewlyDerived::A(entity.id())),
				entity.get::<_NewlyDerived>(),
			);
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

			assert_eq!(
				Some(&_NewlyDerived::A(entity.id())),
				entity.get::<_NewlyDerived>(),
			);
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

			assert_eq!(
				Some(&_AlwaysDerived::A(entity.id())),
				entity.get::<_AlwaysDerived>(),
			);
		}

		#[test]
		fn override_when_derived_already_present() {
			let mut app = setup::<_AlwaysDerived>();

			let entity = app.world_mut().spawn((_Component, _AlwaysDerived::B));

			assert_eq!(
				Some(&_AlwaysDerived::A(entity.id())),
				entity.get::<_AlwaysDerived>(),
			);
		}

		#[test]
		fn insert_again_when_reinserted() {
			let mut app = setup::<_AlwaysDerived>();

			let mut entity = app.world_mut().spawn(_Component);
			entity.remove::<_AlwaysDerived>();
			entity.insert(_Component);

			assert_eq!(
				Some(&_AlwaysDerived::A(entity.id())),
				entity.get::<_AlwaysDerived>(),
			);
		}

		#[test]
		fn prevent_multiple_observers() {
			let mut app = setup::<_AlwaysDerived>();

			app.register_derived_component::<_Component, _AlwaysDerived>();

			assert_count!(1, app.world().iter_entities());
		}
	}
}
