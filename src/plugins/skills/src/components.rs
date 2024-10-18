pub mod combo_node;
pub mod combos;
pub mod combos_time_out;
pub mod inventory;
pub mod lookup;
pub mod queue;
pub mod slots;

pub(crate) mod skill_executer;
pub(crate) mod skill_spawners;

use self::slots::Slots;
use crate::items::{slot_key::SlotKey, Item};
use common::components::Collection;

#[derive(Debug, PartialEq)]
pub(crate) struct LoadModel(pub SlotKey);

pub(crate) type LoadModelsCommand = Collection<LoadModel>;
