use std::sync::Arc;
use status::Status;

/// Represents a node in the behavior tree
pub trait Node<T: Send + Sync + 'static>
{
	/// Ticks the node a single time.
	///
	/// NOTE: Nodes should not automatically reset themselves. This was chosen
	/// in order to remove the need for special "star" nodes. Having the nodes
	/// automatically reset can be simulated using a decorator node.
	fn tick(&mut self, world: &Arc<T>) -> Status;

	/// Resets the node.
	///
	/// This sets the node to a state that is identical to a newly constructed
	/// node.
	fn reset(&mut self);

	/// Gets the current status of the node.
	///
	/// This value will match the return value of the last call to `tick`
	fn status(&self) -> Status;
}
