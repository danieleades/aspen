use node::Node;
use status::Status;

/// Main behavior tree struct
///
/// `T` is required to be `Sync` because, in all likelyhood, it will be used
/// in multiple threads.
pub struct BehaviorTree<T: Sync>
{
	/// Represents the state of the world
	///
	/// A mutable reference to this object will be passed to all nodes when
	/// they are ticked.
	world: T,
	root: Box<Node<T>>
}
impl<T: Sync> BehaviorTree<T>
{
	/// Create a new behavior tree with the given world state object and
	/// root node.
	pub fn new(state: T, root: Box<Node<T>>) -> BehaviorTree<T>
	{
		BehaviorTree { world: state, root: root }
	}

	/// Tick the behavior tree a single time
	pub fn tick(&mut self) -> Status
	{
		(*self.root).tick(&mut self.world)
	}

	/// Reset the tree so that it can be run again
	pub fn reset(&mut self)
	{
		(*self.root).reset()
	}
}
