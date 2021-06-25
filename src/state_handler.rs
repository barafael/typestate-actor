use crate::{
    error::Error,
    state_machine::MessageHandler,
    types::{IncomingMessage, OutgoingMessage, Subscription},
};
use tokio::sync::{broadcast, mpsc};

pub async fn run_until_error(
    mut subscription: mpsc::Receiver<Subscription>,
    application_tx: mpsc::Sender<OutgoingMessage>,
    mut broadcast_rx: broadcast::Receiver<IncomingMessage>,
) -> Result<(), Error> {
    let mut idle = MessageHandler::default();

    loop {
        let sender = subscription
            .recv()
            .await
            .map(|s| s.0)
            .ok_or(Error::SubscriptionListening)?;

        let active = idle.subscribe(sender);

        until_request_event(&mut broadcast_rx).await?;

        let waiting = active.handle_request()?;

        let (message, handler) = waiting.until_request_message().await;

        let message = message?;

        application_tx
            .send(message)
            .await
            .map_err(Error::NoHandlerRegistered)?;

        // Reset handler to idle for next iteration.
        idle = handler;
    }
}

pub async fn until_request_event(
    broadcast_rx: &mut broadcast::Receiver<IncomingMessage>,
) -> Result<(), Error> {
    loop {
        let message = broadcast_rx
            .recv()
            .await
            .map_err(Error::NoSenderRegistered)?;
        if let IncomingMessage::ActionRequest {} = message {
            return Ok(());
        }
    }
}
