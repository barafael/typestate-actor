use crate::{
    error::Error,
    types::{OutgoingMessage, RequestMessage, ResponseMessage},
};
use tokio::sync::oneshot;

/// Idle state.
#[derive(Debug, PartialEq, Eq)]
pub struct Idle;

/// This state holds a sender for sending one request event to the subscribed entity.
#[derive(Debug)]
pub struct Active(oneshot::Sender<RequestMessage>);

/// This state holds a receiver to which the subscribed party can send a response event.
#[derive(Debug)]
pub struct ActionPending(oneshot::Receiver<ResponseMessage>);

/// An actor which handles action requests etc.
/// Parametric over its current state.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct MessageHandler<State> {
    state: State,
}

/// Initial state.
impl Default for MessageHandler<Idle> {
    fn default() -> Self {
        Self { state: Idle }
    }
}

/// Subscribing on an idle handler yields an active handler.
/// This handler listens for an action request.
/// On reception, it transmits the event on the given sender.
impl MessageHandler<Idle> {
    pub fn subscribe(
        self,
        notification_tx: oneshot::Sender<RequestMessage>,
    ) -> MessageHandler<Active> {
        MessageHandler {
            state: Active(notification_tx),
        }
    }
}

/// In the active state, calling this function will notify the subscriber about an action request.
/// It will also hand the subscriber a sender handle for ResponseEvents.
/// After succeeding, this handler will be waiting for a action request response on that channel.
impl MessageHandler<Active> {
    pub fn handle_request(self) -> Result<MessageHandler<ActionPending>, Error> {
        let sender = self.state.0;
        let (response_tx, response_rx) = oneshot::channel();
        sender
            .send(RequestMessage::ActionRequest { response_tx })
            .map_err(Error::SendActionEventFailed)?;
        Ok(MessageHandler {
            state: ActionPending(response_rx),
        })
    }
}

impl MessageHandler<ActionPending> {
    pub async fn until_request_message(
        self,
    ) -> (Result<OutgoingMessage, Error>, MessageHandler<Idle>) {
        let rx = self.state.0;
        let response = match rx.await.map_err(Error::ResponseListening) {
            Ok(m) => m,
            Err(error) => {
                return (Err(error), MessageHandler::default());
            }
        };

        match response {
            ResponseMessage::Completed => (
                Ok(OutgoingMessage::ActionCompleted),
                MessageHandler::default(),
            ),
            ResponseMessage::Failed => {
                (Ok(OutgoingMessage::ActionFailed), MessageHandler::default())
            }
        }
    }
}
