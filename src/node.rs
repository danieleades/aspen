/// Represents a node in the behavior tree
pub trait Node<T: Sync>
{
	/// Ticks the node a single time
	fn tick(&mut self, world: &mut T) -> ::status::Status;
}
