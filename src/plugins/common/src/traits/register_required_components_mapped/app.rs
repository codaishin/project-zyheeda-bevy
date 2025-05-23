use super::RegisterRequiredComponentsMapped;
use crate::traits::try_insert_on::TryInsertOn;
use bevy::prelude::*;

impl RegisterRequiredComponentsMapped for App {
	fn register_required_components_mapped<TComponent, TRequired>(
		&mut self,
		cstr: fn(&TComponent) -> TRequired,
	) -> &mut Self
	where
		TComponent: Component,
		TRequired: Component,
	{
		self.add_observer(insert_required(cstr))
	}
}

#[allow(clippy::type_complexity)]
fn insert_required<TComponent, TRequired>(
	cstr: fn(&TComponent) -> TRequired,
) -> impl Fn(Trigger<OnInsert, TComponent>, Commands, Query<(&TComponent, Option<&TRequired>)>)
where
	TComponent: Component,
	TRequired: Component,
{
	move |trigger, mut commands, components| {
		let entity = trigger.target();
		let Ok((component, None)) = components.get(entity) else {
			return;
		};
		commands.try_insert_on(entity, cstr(component));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_tools::utils::SingleThreadedApp;

	#[derive(Component)]
	struct _Component;

	#[derive(Component, Debug, PartialEq)]
	enum _Required {
		A,
		B,
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.register_required_components_mapped::<_Component, _Required>(|_| _Required::A);

		app
	}

	#[test]
	fn insert_required() {
		let mut app = setup();

		let entity = app.world_mut().spawn(_Component);

		assert_eq!(Some(&_Required::A), entity.get::<_Required>());
	}

	#[test]
	fn do_not_override_when_required_already_present() {
		let mut app = setup();

		let entity = app.world_mut().spawn((_Component, _Required::B));

		assert_eq!(Some(&_Required::B), entity.get::<_Required>());
	}

	#[test]
	fn insert_again_when_reinserted() {
		let mut app = setup();

		let mut entity = app.world_mut().spawn(_Component);
		entity.remove::<_Required>();
		entity.insert(_Component);

		assert_eq!(Some(&_Required::A), entity.get::<_Required>());
	}
}
