//! Nodes that have a single child and modify the behavior of that child in some way.
use node::{Node, Internals};
use status::Status;

/// A node whose status is determined by running a function on its child's status.
///
/// This node will tick its child and then run the supplied function on the
/// child's return status.
///
/// # State
///
/// **Initialized:** Depends on function.
///
/// **Running:** Depends on function.
///
/// **Succeeded:** Depends on function.
///
/// **Failed:** Depends on function.
///
/// # Children
///
/// Takes a single child which is ticked or reset every time the `Decorator` is
/// ticked or reset. The child may be ticked to completion multiple times
/// before the decorator is done.
///
/// # Examples
///
/// A decorator that inverts the return status of its child:
///
/// ```
/// fn invert(s: Status) -> Status
/// {
///     if s == Status::Succeeded { Status::Failed }
///     else if s == Status::Failed { Status::Succeeded }
///     else { s }
/// }
///
/// let child = AlwaysSucceed::new();
/// let node = Decorator::new(child, invert);
/// assert_eq!(node.tick(), Status::Failed);
/// ```
pub struct Decorator
{
	/// Function that is performed on the child's status.
	func: Box<Fn(Status) -> Status>,

	/// Child node.
	child: Node,
}
impl Decorator
{
	/// Creates a new Decorator node with the supplied child node and function
	/// to be run on the child's status.
	pub fn new(child: Node, func: Box<Fn(Status) -> Status>) -> Node
	{
		let internals = Decorator { func: func, child: child };
		Node::new(internals)
	}
}
impl Internals for Decorator
{
	fn tick(&mut self) -> Status
	{
		// If the child has already run, this shouldn't change results since it will
		// just return its last status
		let child_status = self.child.tick();
		(*self.func)(child_status)
	}

	fn reset(&mut self)
	{
		self.child.reset();
	}

	fn children(&self) -> Option<Vec<&Node>>
	{
		Some(vec![&self.child])
	}

	/// Returns the string "Decorator".
	fn type_name(&self) -> &'static str
	{
		"Decorator"
	}
}

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
/// let reset_limit = 5;
/// let child = AlwaysFail::new();
/// let node = Repeat::with_limit(child, reset_limit);
///
/// for _ in 0..reset_limit {
///     assert_eq!(node.tick(), Status::Running);
/// }
/// assert_eq!(node.tick(), Status::Successful);
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
		// Check to see if we have a reset limit
		if let Some(limit) = self.attempt_limit {
			// Make sure we're below the limit
			if self.attempts < limit {
				// If this counts as a reset, add to our counter
				if self.child.status().is_done() {
					self.attempts += 1;
				}

				// Tick the child and return that we're still running
				self.child.tick();
				return Status::Running;
			} else {
				// We've used up all our resets
				return Status::Succeeded;
			}
		}
		else {
			// We're never going to do anything but be running
			self.child.tick();
			return Status::Running;
		}
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

/// A node that repeats its child until the child fails.
///
/// This node will return that it is running until the child fails. It can
/// potentially have a finite reset limit. If the child ever returns that it
/// fails, this node returns that it *succeeds*. If the limit is reached before
/// the child fails, this node *fails*.
///
/// # State
///
/// **Initialized:** Before being ticked after either being created or reset.
///
/// **Running:** While the child node has yet to fail and it is below the reset
/// limit.
///
/// **Succeeded:** Once the child node fails.
///
/// **Failed:** If the reset limit was reached before the child failed.
///
/// # Children
///
/// One, which will be ticked or reset every time the `UntilFail` node is
/// ticked or reset. The child may also be reset multiple times before the parent
/// node is reset or completed.
///
/// # Examples
///
/// A child that will be repeated infinitely until it fails:
///
/// ```
/// let mut a = 0;
/// let child = Condition::new(|| a < 10 );
/// let mut node = UntilFail::new(child);
///
/// for _ in 0..10 {
///     assert_eq!(node.tick(), Status::Running);
///     a += 1;
/// }
///
/// assert_eq!(node.tick(), Status::Succeeded);
/// ```
///
/// An `UntilFail` node will fail if the child doesn't within the limit:
///
/// ```
/// let child = AlwaysSucceed::new();
/// let mut node = UntilFail::with_limit(child, 10);
///
/// for _ in 0..10 {
///     assert_eq!(node.tick(), Status::Running);
///     a += 1;
/// }
///
/// assert_eq!(node.tick(), Status::Failed);
/// ```
pub struct UntilFail
{
	/// Child node.
	child: Node,

	/// Optional number of times to do the reset.
	attempt_limit: Option<u32>,

	/// Number of times the child has been reset.
	attempts: u32,
}
impl UntilFail
{
	/// Creates a new UntilFail node that will keep trying indefinitely.
	pub fn new(child: Node) -> Node
	{
		let internals = UntilFail {
			child: child,
			attempt_limit: None,
			attempts: 0,
		};
		Node::new(internals)
	}

	/// Creates a new UntilFail node that will only retry a specific number of times.
	pub fn with_limit(child: Node, limit: u32) -> Node
	{
		let internals = UntilFail {
			child: child,
			attempt_limit: Some(limit),
			attempts: 0,
		};
		Node::new(internals)
	}
}
impl Internals for UntilFail
{
	fn tick(&mut self) -> Status
	{
		// Check to see if we have a limited number of attempts
		if let Some(limit) = self.attempt_limit {
			// Make sure we're below that limit
			if self.attempts < limit {
				if self.child.status().is_done() {
					self.attempts += 1;
				}

				return if self.child.tick() == Status::Failed {
					Status::Succeeded
				} else { Status::Running };
			}
			else {
				return Status::Failed;
			}
		}
		else {
			self.child.tick();
			return Status::Running;
		}
	}

	fn reset(&mut self)
	{
		// Reset our own status
		self.attempts = 0;

		// Reset the child
		self.child.reset();
	}

	fn children(&self) -> Option<Vec<&Node>>
	{
		Some(vec![&self.child])
	}

	/// Returns the string "UntilFail".
	fn type_name(&self) -> &'static str
	{
		"UntilFail"
	}
}

/// A node that repeats its child until the child succeeds.
///
/// This node will return that it is running until the child succeeds. It can
/// potentially have a finite reset limit. If the child ever returns that it
/// succeeds, this node returns that it *succeeds*. If the limit is reached before
/// the child succeeds, this node *fails*.
///
/// # State
///
/// **Initialized:** Before being ticked after either being created or reset.
///
/// **Running:** While the child node has yet to succeed and it is below the reset
/// limit.
///
/// **Succeeded:** Once the child node succeeds.
///
/// **Failed:** If the reset limit was reached before the child succeeded.
///
/// # Children
///
/// One, which will be ticked or reset every time the `UntilSuccess` node is
/// ticked or reset. The child may also be reset multiple times before the parent
/// node is reset or completed.
///
/// # Examples
///
/// A child that will be repeated infinitely until it succeeds:
///
/// ```
/// let mut a = 0;
/// let child = Condition::new(|| a == 10 );
/// let mut node = UntilSuccess::new(child);
///
/// for _ in 0..10 {
///     assert_eq!(node.tick(), Status::Running);
///     a += 1;
/// }
///
/// assert_eq!(node.tick(), Status::Succeeded);
/// ```
///
/// An `UntilSuccess` node will fail if the child doesn't succeed within the limit:
///
/// ```
/// let child = AlwaysFail::new();
/// let mut node = UntilSuccess::with_limit(child, 10);
///
/// for _ in 0..10 {
///     assert_eq!(node.tick(), Status::Running);
///     a += 1;
/// }
///
/// assert_eq!(node.tick(), Status::Failed);
/// ```
pub struct UntilSuccess
{
	/// Child node.
	child: Node,

	/// Optional number of times to do the reset.
	attempt_limit: Option<u32>,

	/// Number of times the child has been reset.
	attempts: u32,
}
impl UntilSuccess
{
	/// Creates a new `UntilSuccess` node that will keep trying indefinitely.
	pub fn new(child: Node) -> Node
	{
		let internals = UntilSuccess {
			child: child,
			attempt_limit: None,
			attempts: 0,
		};
		Node::new(internals)
	}

	/// Creates a new `UntilSuccess` node that will only retry a specific number of times.
	pub fn with_limit(child: Node, limit: u32) -> Node
	{
		let internals = UntilSuccess {
			child: child,
			attempt_limit: Some(limit),
			attempts: 0,
		};
		Node::new(internals)
	}
}
impl Internals for UntilSuccess
{
	fn tick(&mut self) -> Status
	{
		// Check to see if we have a limited number of attempts
		if let Some(limit) = self.attempt_limit {
			// Make sure we're below that limit
			if self.attempts < limit {
				if self.child.status().is_done() {
					self.attempts += 1;
				}

				return if self.child.tick() == Status::Succeeded {
					Status::Succeeded
				} else { Status::Running };
			}
			else {
				return Status::Failed;
			}
		}
		else {
			self.child.tick();
			return Status::Running;
		}
	}

	fn reset(&mut self)
	{
		// Reset our own status
		self.attempts = 0;

		// Reset the child
		self.child.reset();
	}

	fn children(&self) -> Option<Vec<&Node>>
	{
		Some(vec![&self.child])
	}

	/// Returns the string "UntilSuccess".
	fn type_name(&self) -> &'static str
	{
		"UntilSuccess"
	}
}

#[cfg(test)]
mod test
{
	use status::Status;
	use std_nodes::*;

	fn rotate(s: Status) -> Status
	{
		match s {
			Status::Initialized => Status::Running,
			Status::Running => Status::Succeeded,
			Status::Succeeded => Status::Failed,
			Status::Failed => Status::Initialized,
		}
	}

	#[test]
	fn check_decorator()
	{
		// Test the first rotation
		let suc_child = YesTick::new(Status::Succeeded);
		let mut suc_dec = Decorator::new(suc_child, Box::new(rotate));
		let suc_status = suc_dec.tick();
		drop(suc_dec);
		assert_eq!(suc_status, rotate(Status::Succeeded));

		// Test the second rotation
		let run_child = YesTick::new(Status::Running);
		let mut run_dec = Decorator::new(run_child, Box::new(rotate));
		let run_status = run_dec.tick();
		drop(run_dec);
		assert_eq!(run_status, rotate(Status::Running));

		// Test the final rotation
		let fail_child = YesTick::new(Status::Failed);
		let mut fail_dec = Decorator::new(fail_child, Box::new(rotate));
		let fail_status = fail_dec.tick();
		drop(fail_dec);
		assert_eq!(fail_status, rotate(Status::Failed));
	}

	#[test]
	fn check_reset()
	{
		// No good way to test ticking indefinitely, so we'll tick a
		// specified number of times
		let child = CountedTick::new(Status::Succeeded, 5, true);
		let mut reset = Reset::with_limit(child, 5);

		// Tick it five times
		let mut status = Status::Running;
		for _ in 0..5 {
			status = reset.tick();
		}

		// Drop the node so the testing nodes can panic
		drop(reset);

		// Now make sure we got the right output
		assert_eq!(status, Status::Succeeded);
	}

	#[test]
	fn check_retry()
	{
		// We can test to make sure that the "indefinite" only ticks while failed
		let child1 = CountedTick::new(Status::Succeeded, 1, true);
		let mut retry1 = Retry::new(child1);
		let mut status1 = Status::Running;
		while status1 == Status::Running { status1 = retry1.tick(); };
		drop(retry1);
		assert_eq!(status1, Status::Succeeded);

		// No good way to test infinite retrying, so use a limited number
		let child2 = CountedTick::new(Status::Failed, 5, true);
		let mut retry2 = Retry::with_limit(child2, 5);
		let mut status2 = Status::Running;
		for _ in 0..5 { status2 = retry2.tick(); }
		drop(retry2);
		assert_eq!(status2, Status::Failed);
	}
}
