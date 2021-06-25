use crate::{
    error::Error,
    types::{IncomingMessage, OutgoingMessage},
};
use tokio::sync::{broadcast, mpsc};

pub async fn run_until_error(
    incoming_tx: mpsc::Sender<OutgoingMessage>,
    mut broadcast_rx: broadcast::Receiver<IncomingMessage>,
) -> Result<(), Error> {
    loop {
        let challenge = until_challenge_event(&mut broadcast_rx).await?;

        let response = handle_challenge(challenge);

        incoming_tx
            .send(response)
            .await
            .map_err(Error::NoHandlerRegistered)?;
    }
}

async fn until_challenge_event(
    broadcast_rx: &mut broadcast::Receiver<IncomingMessage>,
) -> Result<u64, Error> {
    loop {
        let message = broadcast_rx
            .recv()
            .await
            .map_err(Error::NoSenderRegistered)?;
        if let IncomingMessage::Challenge(challenge) = message {
            break Ok(challenge);
        }
    }
}

fn handle_challenge(challenge: u64) -> OutgoingMessage {
    OutgoingMessage::ChallengeResponse(challenge)
}
