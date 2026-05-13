use crate::GetNormalizedName;
use bevy::{
	ecs::system::{IntoObserverSystem, StaticSystemParam},
	prelude::*,
};
use common::traits::{
	accessors::get::GetContextMut,
	handles_interactive::{Interactive, SetInteractive, SetInteractiveRole},
	thread_safe::ThreadSafe,
};
use std::collections::HashMap;
use zyheeda_core::strings::normalized_name::NormalizedName;

impl<T> IdentifyInteractive for T where
	T: for<'c> GetContextMut<SetInteractive, TContext<'c>: SetInteractiveRole> + ThreadSafe
{
}

pub(crate) trait IdentifyInteractive:
	for<'c> GetContextMut<SetInteractive, TContext<'c>: SetInteractiveRole> + ThreadSafe
{
	fn identify_interactive(
		lookup: &[(GetNormalizedName, Interactive)],
	) -> impl IntoObserverSystem<Add, Name, ()> {
		let lookup = lookup
			.iter()
			.map(|(n, i)| (n(), *i))
			.collect::<HashMap<NormalizedName, Interactive>>();

		#[rustfmt::skip]
		let observer = move |
			added_name: On<Add, Name>,
		  names: Query<&Name>,
		  mut interactive_param: StaticSystemParam<Self>
		| {
			let entity = added_name.entity;

			let Ok(name) = names.get(entity) else {
				return;
			};

			let Some(role) = lookup.get(&NormalizedName::from(name.as_str())) else {
				return;
			};

			let key = SetInteractive { entity };
			let Some(mut ctx) = Self::get_context_mut(&mut interactive_param, key) else {
				return;
			};

			ctx.set_interactive_role(*role);
		};

		IntoObserverSystem::into_system(observer)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::handles_interactive::Door;
	use test_case::test_case;
	use testing::SingleThreadedApp;
	use zyheeda_core::strings::normalized_name::NormalizedName;
	#[derive(Component, Debug, PartialEq)]
	struct _Interactive(Option<Interactive>);

	impl SetInteractiveRole for _Interactive {
		fn set_interactive_role(&mut self, role: Interactive) {
			self.0 = Some(role);
		}
	}

	fn setup(names: &[(GetNormalizedName, Interactive)]) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_observer(Query::<&mut _Interactive>::identify_interactive(names));

		app
	}

	#[test_case(Interactive::Door(Door::SlideDoor); "slide door")]
	fn set_role(role: Interactive) {
		let mut app = setup(&[(|| NormalizedName::from("aa"), role)]);

		let entity = app
			.world_mut()
			.spawn((Name::from("aa"), _Interactive(None)));

		assert_eq!(
			Some(&_Interactive(Some(role))),
			entity.get::<_Interactive>(),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup(&[(
			|| NormalizedName::from("aa"),
			Interactive::Door(Door::SlideDoor),
		)]);

		let mut entity = app
			.world_mut()
			.spawn((Name::from("aa"), _Interactive(None)));
		entity.insert(_Interactive(None));
		entity.insert(Name::from("aa"));

		assert_eq!(Some(&_Interactive(None)), entity.get::<_Interactive>());
	}
}
