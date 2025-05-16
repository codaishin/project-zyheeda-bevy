use crate::components::primer_panel::PrimerPanel;
use common::tools::action_key::user_input::UserInput;

impl<T> PrimerPanel<T>
where
	T: Into<UserInput>,
{
	pub(crate) fn add_ui_input_primer() {}
}
