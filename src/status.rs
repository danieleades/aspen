#[cfg(feature = "lcm")]
use node_message::node_msg;

/// Represents the status of a given node in the behavior tree.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Status
{
	/// A `Node` with this status is one that has not been ticked since either
	/// its creation or since it was last reset.
	Initialized,
	/// Represents that a `Node` is currently executing its logic.
	Running,
	/// The status of a `Node` which as been ticked to completion and
	/// successfully executed its logic.
	Succeeded,
	/// That status of a `Node` which has been ticked to completion and failed
	/// to execute its logic.
	Failed
}
impl Status
{
	/// Returns `true` if the `Status` is one where execution has finished.
	///
	/// Execution is considered to be done if it is either `Succeeded` or
	/// `Failed`.
	pub fn is_done(&self) -> bool
	{
		*self == Status::Succeeded || *self == Status::Failed
	}
}

impl From<Status> for i8
{
	#[cfg(feature = "lcm")]
	fn from(s: Status) -> i8
	{
		// A simple `s as i8` would probably do the correct thing, but using
		// the constants from the message file will make sure that the values
		// are correct even if a new status is implemented (e.g., cancel) or
		// someone swaps the order.
		match s {
			Status::Initialized => node_msg::INITIALIZED,
			Status::Running => node_msg::RUNNING,
			Status::Succeeded => node_msg::SUCCEEDED,
			Status::Failed => node_msg::FAILED,
		}
	}

	#[cfg(not(feature = "lcm"))]
	fn from(s: Status) -> i8
	{
		s as i8
	}
}
