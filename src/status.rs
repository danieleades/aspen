/// Represents the status of a given node in the behavior tree
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Status
{
	/// Represents a node that has yet to be run
	///
	/// If there wasn't a need to distinguish between running and yet-to-run
	/// nodes in the aspen-gui, I would not have this here. However, there is
	/// so this extra `Status` is necessary
	Initialized,
	/// Represents a node that is currently running
	Running,
	/// Represents a node that succeeded
	Succeeded,
	/// Represents a node that has failed
	Failed
}
impl Status
{
	/// Returns `true` if the `Status` is one where execution has finished
	pub fn is_done(&self) -> bool
	{
		*self == Status::Succeeded || *self == Status::Failed
	}
}
