/// Represents the status of a given node in the behavior tree
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Status
{
	/// Represents a node that succeeded
	Succeeded,
	/// Represents a node that is currently running or has yet to run
	Running,
	/// Represents a node that has failed
	Failed
}
