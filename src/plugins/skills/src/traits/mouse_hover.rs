use super::WithComponent;
use crate::skill::Outdated;
use bevy::ecs::{component::Component, entity::Entity, system::Query};
use common::resources::{ColliderInfo, MouseHover};

impl<T: Component + Copy> WithComponent<T> for MouseHover {
	fn with_component(&self, query: &Query<&T>) -> Option<ColliderInfo<Outdated<T>>> {
		let outdated_component = |entity: Entity| {
			Some(Outdated {
				entity,
				component: *query.get(entity).ok()?,
			})
		};

		self.0.clone().and_then(|mouse_hover| {
			Some(ColliderInfo {
				collider: outdated_component(mouse_hover.collider)?,
				root: mouse_hover.root.and_then(outdated_component),
			})
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::system::Commands,
	};

	#[derive(Component, Debug, PartialEq)]
	struct _Result<T: Component>(Option<ColliderInfo<Outdated<T>>>);

	#[derive(Component, Debug, PartialEq, Clone, Copy)]
	struct _Comp(usize);

	fn run_with_component<T: Component + Copy>(src: MouseHover) -> impl Fn(Commands, Query<&T>) {
		move |mut commands, query| {
			let result = _Result(src.with_component(&query));
			commands.spawn(result);
		}
	}

	fn get_result<T: Component>(app: &App) -> &_Result<T> {
		app.world
			.iter_entities()
			.find_map(|e| e.get::<_Result<T>>())
			.unwrap()
	}

	#[test]
	fn get_collider_with_no_root() {
		let mut app = App::new();

		let info = ColliderInfo {
			collider: app.world.spawn(_Comp(42)).id(),
			root: None,
		};
		app.add_systems(
			Update,
			run_with_component::<_Comp>(MouseHover(Some(info.clone()))),
		);
		app.update();

		assert_eq!(
			&_Result(Some(ColliderInfo {
				collider: Outdated {
					entity: info.collider,
					component: _Comp(42),
				},
				root: None
			})),
			get_result::<_Comp>(&app)
		);
	}

	#[test]
	fn get_collider_with_root() {
		let mut app = App::new();

		let info = ColliderInfo {
			collider: app.world.spawn(_Comp(42)).id(),
			root: Some(app.world.spawn(_Comp(44)).id()),
		};
		app.add_systems(
			Update,
			run_with_component::<_Comp>(MouseHover(Some(info.clone()))),
		);
		app.update();

		assert_eq!(
			&_Result(Some(ColliderInfo {
				collider: Outdated {
					entity: info.collider,
					component: _Comp(42),
				},
				root: Some(Outdated {
					entity: info.root.unwrap(),
					component: _Comp(44),
				})
			})),
			get_result::<_Comp>(&app)
		);
	}

	#[test]
	fn get_none_when_src_none() {
		let mut app = App::new();

		app.world.spawn(_Comp(42));
		app.add_systems(Update, run_with_component::<_Comp>(MouseHover(None)));
		app.update();

		assert_eq!(&_Result(None), get_result::<_Comp>(&app));
	}

	#[test]
	fn get_none_when_no_component() {
		let mut app = App::new();

		let info = ColliderInfo {
			collider: app.world.spawn_empty().id(),
			root: None,
		};
		app.add_systems(Update, run_with_component::<_Comp>(MouseHover(Some(info))));
		app.update();

		assert_eq!(&_Result(None), get_result::<_Comp>(&app));
	}
}
