use thiserror::Error;
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::types::{OutgoingMessage, RequestMessage};

#[derive(Debug, Error)]
pub enum Error {
    /// Failed to send action event.
    #[error("Failed to send action event: {0:?}")]
    SendActionEventFailed(RequestMessage),

    /// No sender registered while in listening state.
    #[error("No sender registered while in listening state")]
    NoSenderRegistered(#[source] broadcast::error::RecvError),

    /// Received action request but no handler is registered.
    #[error("Received action request but no handler is registered")]
    NoHandlerRegistered(#[source] mpsc::error::SendError<OutgoingMessage>),

    /// Received action request but action already pending.
    #[error("Received action request but action already pending")]
    ResponseListening(oneshot::error::RecvError),

    /// Receiving the subscription of a listener failed.
    #[error("Receiving the subscription of a listener failed")]
    SubscriptionListening,
}
