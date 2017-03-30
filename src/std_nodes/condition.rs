//! Nodes which query the state of the world.
use node::{Node, Internals};
use status::Status;

/// A node whose status is determined by a function.
///
/// When ticked, this node will run the supplied function to determine its
/// return value. If the function returns `true`, the node is considered
/// successful - it is considered failed otherwise. Note that this means the node
/// will never be in a running state. As such, all supplied functions should be
/// able to be completed within a single tick's amount of time.
///
/// # State
///
/// **Initialized:** Before being ticked after either being created or reset.
///
/// **Running:** Never.
///
/// **Succeeded:** If the supplied function returns `true`.
///
/// **Failed:** If the supplied function returns `false`.
///
/// # Children
///
/// None
///
/// # Examples
///
/// A condition node that checks whether or not it is possible to subtract two
/// unsigned integers:
///
/// ```
/// # use std_nodes::*;
/// # use status::Status;
/// let a: u32 = 10;
/// let b: u32 = 100;
///
/// let mut node = Condition::new(|| a > b );
/// assert_eq!(node.tick(), Status::Failed);
/// ```
pub struct Condition
{
	/// Function that is performed to determine the node's status
	///
	/// A return value of `true` means success and a return value of `false`
	/// means failure.
	func: Box<Fn() -> bool>,
}
impl Condition
{
	/// Constructs a new Condition node that will run the given function.
	pub fn new<F: Fn() -> bool + 'static>(func: F) -> Node
	{
		let internals = Condition { func: Box::new(func) };
		Node::new(internals)
	}
}
impl Internals for Condition
{
	fn tick(&mut self) -> Status
	{
		// Otherwise, run the function
		if (*self.func)() {
			Status::Succeeded
		} else {
			Status::Failed
		}
	}

	fn reset(&mut self)
	{
		// No-op
	}

	/// Returns the string "Condition".
	fn type_name(&self) -> &'static str
	{
		"Condition"
	}
}

#[cfg(test)]
mod test
{
	use status::Status;
	use std_nodes::*;

	#[test]
	fn failure()
	{
		let mut cond = Condition::new(|| false);
		assert_eq!(cond.tick(), Status::Failed);
	}

	#[test]
	fn success()
	{
		let mut cond = Condition::new(|| true);
		assert_eq!(cond.tick(), Status::Succeeded);
	}
}
