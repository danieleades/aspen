/// Represents the status of a given node in the behavior tree
pub enum Status
{
	/// Represents a node that succeeded
	Succeded,
	/// Represents a node that is currently running
	Running,
	/// Represents a node that has failed
	Failed(Box<::std::error::Error>)
}
