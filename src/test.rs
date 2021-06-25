use crate::{
    challenge_handler, state_handler,
    state_machine::MessageHandler,
    types::{IncomingMessage, OutgoingMessage, RequestMessage, ResponseMessage, Subscription},
};
use tokio::sync::{broadcast, mpsc, oneshot};

#[tokio::test]
async fn message_handler_state_machine() {
    let handler = MessageHandler::default();
    let (tx, mut rx) = oneshot::channel();
    let handler = handler.subscribe(tx);
    assert!(rx.try_recv().is_err());
    let handler = handler.handle_request().unwrap();
    let event = rx.try_recv().unwrap();
    let tx = match event {
        RequestMessage::ActionRequest { response_tx } => response_tx,
    };

    let handle = tokio::spawn(handler.until_request_message());

    tx.send(ResponseMessage::Completed).unwrap();

    let (message, handler) = tokio::join!(handle).0.unwrap();

    assert_eq!(message.unwrap(), OutgoingMessage::ActionCompleted {},);

    assert_eq!(handler, MessageHandler::default(),);

    // Test the cycle can start anew.
    let (tx, _) = oneshot::channel();
    let _handler = handler.subscribe(tx);
}

#[tokio::test]
async fn message_handler_run_till_error() {
    let (sub_tx, sub_rx) = mpsc::channel(1);
    let (application_tx, mut application_rx) = mpsc::channel(16);
    let (broadcast_tx, broadcast_rx) = broadcast::channel(1);

    let _task_handle = tokio::spawn(state_handler::run_until_error(
        sub_rx,
        application_tx,
        broadcast_rx,
    ));

    let (handle_tx, handle_rx) = oneshot::channel();
    let subscription = Subscription(handle_tx);

    sub_tx.send(subscription).await.unwrap();

    let action_request = IncomingMessage::ActionRequest {};
    broadcast_tx.send(action_request).unwrap();

    let event = handle_rx.await.unwrap();

    let response_tx = match event {
        RequestMessage::ActionRequest { response_tx } => response_tx,
    };

    let response_event = ResponseMessage::Completed;
    response_tx.send(response_event).unwrap();

    let app_msg = application_rx.recv().await.unwrap();
    assert_eq!(OutgoingMessage::ActionCompleted {}, app_msg);
}

#[tokio::test]
async fn run_till_error_challenge() {
    let (incoming_tx, mut incoming_rx) = mpsc::channel(1);
    let (outgoing_tx, outgoing_rx) = broadcast::channel(1);

    let _task_handle = tokio::spawn(challenge_handler::run_until_error(incoming_tx, outgoing_rx));

    let ping = IncomingMessage::Challenge(12);
    outgoing_tx.send(ping).unwrap();

    let response = incoming_rx.recv().await.unwrap();
    assert_eq!(OutgoingMessage::ChallengeResponse(12), response);
}
