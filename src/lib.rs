#[cfg(test)]
mod test;

/// Represents the status of a given node in the behavior tree
pub enum Status
{
	Succeeded,
	Running,
	Failed(Box<std::error::Error>)
}

/// Represents a node in the behavior tree
trait Node<T: Sync>
{
	/// Ticks the node a single time
	fn tick(&mut self, world: &mut T) -> Status;
}

/// Main behavior tree struct.
///
/// `T` is required to be `Sync` because, in all likelyhood, it will be used
/// in multiple threads.
pub struct BehaviorTree<T: Sync>
{
	/// The item that is passed to each leaf node
	world: T
}
impl<T: Sync> BehaviorTree<T>
{
	/// Create a new behavior tree with the given world state object
	pub fn new(state: T) -> BehaviorTree<T>
	{
		BehaviorTree { world: state }
	}
}
impl<T: Default + Sync> Default for BehaviorTree<T>
{
	/// Creates a behavior tree with a default world state object
	fn default() -> BehaviorTree<T>
	{
		BehaviorTree { world: Default::default() }
	}
}
