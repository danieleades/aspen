use node::{Node, Internals};
use status::Status;

/// A node that will repeat its child a specific number of times, possibly infinite.
///
/// A repeat node will report that it is running until its child node has been
/// run to completion the specified number of times, upon which it will be
/// considered successful. This could also be an infinite number, in which case
/// this node will always be considered running.
///
/// # State
///
/// **Initialized:** Before being ticked after either being reset or created.
///
/// **Running:** Until the child node has been reset the specified number of
/// times. If there is no limit, always.
///
/// **Succeeded:** Once the child has been reset the specified number of times.
/// If there is no limit, never.
///
/// **Failed:** Never.
///
/// # Children
///
/// One. It is ticked or reset whenever the repeat node is ticked or reset. It
/// also may be reset multiple times before the repeat node is reset or completed.
///
/// # Examples
///
/// Force the child to be reset a specific number of times:
///
/// ```
/// # use aspen::std_nodes::*;
/// # use aspen::Status;
/// let run_limit = 5;
/// let child = AlwaysFail::new();
/// let mut node = Repeat::with_limit(child, run_limit);
///
/// // Subtract one since there is a run in the assert
/// for _ in 0..(run_limit - 1) {
///     assert_eq!(node.tick(), Status::Running);
/// }
/// assert_eq!(node.tick(), Status::Succeeded);
/// ```
pub struct Repeat
{
	/// Child node.
	child: Node,

	/// Optional number of times to do the reset.
	attempt_limit: Option<u32>,

	/// Number of times the child has been reset.
	attempts: u32,
}
impl Repeat
{
	/// Creates a new Repeat node that will repeat forever.
	pub fn new(child: Node) -> Node
	{
		let internals = Repeat {
			child: child,
			attempt_limit: None,
			attempts: 0,
		};
		Node::new(internals)
	}

	/// Creates a new Repeat node that will only repeat a limited number of times.
	///
	/// The limit specifies the number of times this node can be run. A limit
	/// of zero means that the node will instantly succeed.
	pub fn with_limit(child: Node, limit: u32) -> Node
	{
		let internals = Repeat {
			child: child,
			attempt_limit: Some(limit),
			attempts: 0,
		};
		Node::new(internals)
	}
}
impl Internals for Repeat
{
	fn tick(&mut self) -> Status
	{
		// Take care of the infinite version so we don't have to worry
		if self.attempt_limit.is_none() {
			self.child.tick();
			return Status::Running;
		}

		// We're using the finite version
		let limit = self.attempt_limit.unwrap();
		let child_status = self.child.tick();

		if child_status.is_done() {
			self.attempts += 1;
			if self.attempts < limit {
				return Status::Running;
			}
			else {
				return Status::Succeeded;
			}
		}

		// We're still running
		Status::Running
	}

	fn reset(&mut self)
	{
		// Reset our attempt count
		self.attempts = 0;

		// Reset the child
		self.child.reset();
	}

	fn children(&self) -> Option<Vec<&Node>>
	{
		Some(vec![&self.child])
	}

	/// Returns the string "Repeat".
	fn type_name(&self) -> &'static str
	{
		"Repeat"
	}
}

#[cfg(test)]
mod test
{
	use status::Status;
	use std_nodes::*;

	#[test]
	fn repeat_infinite()
	{
		// No good way to test the infinite one
		let limit = 5;
		let child = CountedTick::new(Status::Failed, limit, true);
		let mut node = Repeat::with_limit(child, limit);
		for _ in 0..(limit - 1) {
			assert_eq!(node.tick(), Status::Running);
		}
		let status = node.tick();
		drop(node);
		assert_eq!(status, Status::Succeeded);
	}
}
