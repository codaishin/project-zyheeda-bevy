use crate::traits::inspect_able::InspectMarker;

pub struct GridCellDistance;

impl InspectMarker for GridCellDistance {
	type TFieldRef<'a> = f32;
}
