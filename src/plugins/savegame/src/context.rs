use bevy::prelude::*;

use crate::traits::execute_save::ExecuteSave;

#[derive(Debug, PartialEq, Default)]
pub struct SaveContext;

impl ExecuteSave for SaveContext {
	fn execute_save<T>(&mut self, _: EntityRef) {}
}
