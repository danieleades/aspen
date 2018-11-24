//! Nodes which query the state of the world.
use crate::node::{Node, Tickable};
use crate::status::Status;

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
/// # use aspen::node::Tickable;
/// const CHECK_VALUE: u32 = 100;
/// let mut state = 10u32;
///
/// let mut node = Condition::new(|s| *s > CHECK_VALUE );
/// assert_eq!(node.tick(&mut state), Status::Failed);
/// ```
pub struct Condition<'a, S>
{
	/// Function that is performed to determine the node's status
	///
	/// A return value of `true` means success and a return value of `false`
	/// means failure.
	func: Box<Fn(&S) -> bool + 'a>,
}
impl<'a, S> Condition<'a, S>
	where S: 'a
{
	/// Constructs a new Condition node that will run the given function.
	pub fn new<F>(func: F) -> Node<'a, S>
		where F: Fn(&S) -> bool + 'a
	{
		let internals = Condition { func: Box::new(func) };
		Node::new(internals)
	}
}
impl<'a, S> Tickable<S> for Condition<'a, S>
{
	fn tick(&mut self, world: &mut S) -> Status
	{
		// Otherwise, run the function
		if (*self.func)(world) {
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
/// # fn test(_: &()) -> bool { false }
/// # fn main() {
/// let condition = Condition!{ |s| test(s) };
/// # }
/// ```
#[macro_export]
macro_rules! Condition
{
	( $e:expr ) => {
		$crate::std_nodes::Condition::new($e)
	}
}

#[cfg(test)]
mod tests
{
	use crate::status::Status;
	use crate::std_nodes::*;
	use crate::node::Tickable;

	#[test]
	fn failure()
	{
		let mut cond = Condition::new(|_| false);
		assert_eq!(cond.tick(&mut ()), Status::Failed);
	}

	#[test]
	fn success()
	{
		let mut cond = Condition::new(|_| true);
		assert_eq!(cond.tick(&mut ()), Status::Succeeded);
	}
}
