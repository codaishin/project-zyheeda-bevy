pub mod movement;
pub mod shoot_gun;

use bevy::{ecs::system::EntityCommands, prelude::*};

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum MovementMode {
	#[default]
	Walk,
	Run,
}

pub type InsertComponentFn = fn(&mut EntityCommands, Ray);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Behavior {
	pub insert_fn: InsertComponentFn,
}

impl Behavior {
	pub fn insert_into(&self, entity: &mut EntityCommands, ray: Ray) {
		(self.insert_fn)(entity, ray)
	}
}
