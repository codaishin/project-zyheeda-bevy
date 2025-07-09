use crate::{context::SaveContext, file_io::FileIO};
use bevy::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(Resource, Debug)]
pub(crate) struct Inspector<TIO = FileIO> {
	pub(crate) quick_save: Arc<Mutex<SaveContext<TIO>>>,
}
