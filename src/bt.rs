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
