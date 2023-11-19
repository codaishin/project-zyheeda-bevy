use bevy::{ecs::system::EntityCommands, prelude::Component};
use std::marker::PhantomData;

pub struct Slow;

pub struct Fast;

pub struct Idle;

pub struct Shoot;

#[derive(PartialEq, Debug)]
pub struct Left;

#[derive(PartialEq, Debug)]
pub struct Right;

#[derive(PartialEq, Debug)]
pub struct HandGun;

#[derive(Component)]
pub struct Marker<T> {
	phantom_data: PhantomData<T>,
}

impl<T: Send + Sync + 'static> Marker<T> {
	pub fn new() -> Self {
		Self {
			phantom_data: PhantomData,
		}
	}

	pub fn commands() -> Markers {
		Markers {
			insert_fn: insert_marker::<T>,
			remove_fn: remove_marker::<T>,
		}
	}
}

impl<T: Send + Sync + 'static> Default for Marker<T> {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(PartialEq, Debug, Clone)]
pub struct Markers {
	insert_fn: fn(&mut EntityCommands),
	remove_fn: fn(&mut EntityCommands),
}

fn do_nothing(_: &mut EntityCommands) {}

impl Default for Markers {
	fn default() -> Self {
		Self {
			insert_fn: do_nothing,
			remove_fn: do_nothing,
		}
	}
}

impl Markers {
	pub fn insert_to(&self, entity: &mut EntityCommands) {
		(self.insert_fn)(entity)
	}

	pub fn remove_from(&self, entity: &mut EntityCommands) {
		(self.remove_fn)(entity)
	}
}

fn insert_marker<T: Send + Sync + 'static>(entity: &mut EntityCommands) {
	entity.insert(Marker::<T>::new());
}

fn remove_marker<T: Send + Sync + 'static>(entity: &mut EntityCommands) {
	entity.remove::<Marker<T>>();
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::{App, Commands, Entity, Update};

	fn insert(marker_commands: Markers, entity: Entity) -> impl Fn(Commands) {
		move |mut commands| {
			let mut entity = commands.entity(entity);
			marker_commands.insert_to(&mut entity);
		}
	}

	#[test]
	fn insert_marker() {
		let mut app = App::new();
		let commands = Marker::<(f32, u32)>::commands();
		let entity = app.world.spawn(()).id();

		app.add_systems(Update, insert(commands, entity));
		app.update();

		let entity = app.world.entity(entity);

		assert!(entity.contains::<Marker<(f32, u32)>>());
	}

	fn remove(marker_commands: Markers, entity: Entity) -> impl Fn(Commands) {
		move |mut commands| {
			let mut entity = commands.entity(entity);
			marker_commands.remove_from(&mut entity);
		}
	}

	#[test]
	fn remove_marker() {
		let mut app = App::new();
		let commands = Marker::<(f32, u32)>::commands();
		let entity = app.world.spawn(Marker::<(f32, u32)>::new()).id();

		app.add_systems(Update, remove(commands, entity));
		app.update();

		let entity = app.world.entity(entity);

		assert!(!entity.contains::<Marker<(f32, u32)>>());
	}
}
