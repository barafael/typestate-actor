use tokio::sync::oneshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncomingMessage {
    ActionRequest,
    Challenge(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutgoingMessage {
    ActionCompleted,
    ActionFailed,
    ChallengeResponse(u64),
}

/// Messages related to action requests.
#[derive(Debug)]
pub enum RequestMessage {
    /// A request for an action was received.
    ActionRequest {
        response_tx: oneshot::Sender<ResponseMessage>,
    },
}

/// Response messages for action requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResponseMessage {
    /// The action was completed successfully.
    Completed,
    /// The action failed.
    Failed,
}

#[derive(Debug)]
pub struct Subscription(pub(crate) oneshot::Sender<RequestMessage>);
