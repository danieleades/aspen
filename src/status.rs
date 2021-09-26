/// Represents the status of a given node in the behavior tree.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Status {
    /// Represents that a `Node` is currently executing its logic.
    Running,
    /// The status of a `Node` which as been ticked to completion and
    /// successfully executed its logic.
    Succeeded,
    /// That status of a `Node` which has been ticked to completion and failed
    /// to execute its logic.
    Failed,
}
impl Status {
    /// Returns `true` if the `Status` is one where execution has finished.
    ///
    /// Execution is considered to be done if it is either `Succeeded` or
    /// `Failed`.
    #[must_use]
    pub fn is_done(&self) -> bool {
        *self == Status::Succeeded || *self == Status::Failed
    }
}
