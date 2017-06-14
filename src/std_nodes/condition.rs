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
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// const FIRST: u32 = 10;
/// const SECOND: u32 = 100;
///
/// let mut node = Condition::new(|| FIRST > SECOND );
/// assert_eq!(node.tick(), Status::Failed);
/// ```
pub struct Condition<'a>
{
	/// Function that is performed to determine the node's status
	///
	/// A return value of `true` means success and a return value of `false`
	/// means failure.
	func: Box<Fn() -> bool + 'a>,
}
impl<'a> Condition<'a>
{
	/// Constructs a new Condition node that will run the given function.
	pub fn new<F: Fn() -> bool + 'a>(func: F) -> Node<'a>
	{
		let internals = Condition { func: Box::new(func) };
		Node::new(internals)
	}
}
impl<'a> Internals for Condition<'a>
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

/// Convenience macro for creating Condition nodes.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate aspen;
/// # fn main() {
/// # let (a, b) = (12, 13);
/// let condition = Condition!{ a < b };
/// # }
/// ```
#[macro_export]
macro_rules! Condition
{
	( $( $e:expr );* ) => {
		$crate::std_nodes::Condition::new(|| { $( $e );* })
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
