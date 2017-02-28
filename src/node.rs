use status::Status;

/// Represents a node in the behavior tree
pub trait Node<T: Sync>
{
	/// Ticks the node a single time
	fn tick(&mut self, world: &mut T) -> Status;

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
