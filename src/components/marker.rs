use bevy::{ecs::system::EntityCommands, prelude::Component};
use std::marker::PhantomData;

pub struct Walk;

pub struct Run;

pub struct Idle;

pub struct Shoot;

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

	pub fn commands() -> MarkerCommands {
		MarkerCommands {
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

#[derive(PartialEq, Debug)]
pub struct MarkerCommands {
	insert_fn: fn(&mut EntityCommands),
	remove_fn: fn(&mut EntityCommands),
}

impl MarkerCommands {
	pub fn insert_marker_on(&self, entity: &mut EntityCommands) {
		(self.insert_fn)(entity)
	}

	pub fn remove_marker_on(&self, entity: &mut EntityCommands) {
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

	fn insert(marker_commands: MarkerCommands, entity: Entity) -> impl Fn(Commands) {
		move |mut commands| {
			let mut entity = commands.entity(entity);
			marker_commands.insert_marker_on(&mut entity);
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

	fn remove(marker_commands: MarkerCommands, entity: Entity) -> impl Fn(Commands) {
		move |mut commands| {
			let mut entity = commands.entity(entity);
			marker_commands.remove_marker_on(&mut entity);
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
