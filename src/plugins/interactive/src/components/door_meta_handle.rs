use crate::assets::door_meta::DoorMeta;
use bevy::prelude::*;
use common::traits::accessors::get::View;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct DoorMetaHandle(pub(crate) Handle<DoorMeta>);

impl View<Handle<DoorMeta>> for DoorMetaHandle {
	fn view(&self) -> &'_ Handle<DoorMeta> {
		&self.0
	}
}

impl From<&'_ DoorMetaHandle> for AssetId<DoorMeta> {
	fn from(DoorMetaHandle(handle): &'_ DoorMetaHandle) -> Self {
		handle.id()
	}
}
