use crate::context::SaveBuffer;
use bevy::prelude::*;
use serde_json::Error;

pub(crate) trait BufferEntityComponent {
	fn buffer_component(
		&self,
		buffer: &mut SaveBuffer,
		entity: EntityRef,
	) -> Result<(), Error>;
}
