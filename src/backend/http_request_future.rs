use std::cmp::PartialEq;
use std::fmt;
use std::fmt::Formatter;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

/// Error specifiers for HTTP request future errors.
pub enum HttpRequestFutureErrorKind {
    Unknown
}

/// Error handler for HTTP request futures.
pub struct HttpRequestFutureError {
    kind: HttpRequestFutureErrorKind,
}

impl fmt::Display for HttpRequestFutureError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.kind {
            HttpRequestFutureErrorKind::Unknown => {
                write!(f, "Unknown async error within HTTP future handling!")
            }
        }
    }
}

/// The state that the HTTP request future operation is at.
#[derive(PartialEq)]
pub(in crate) enum HttpRequestFutureState {
    Uninitialized,
    Errored,
    InProgress,
    Completed,
}

/// The shared HTTP request structure.
///
/// This structure is used to create mutable references to active HTTP requests to handle their
/// parsing.
pub(in crate) struct SharedHttpRequest {
    state: HttpRequestFutureState,
    string_data: String,
    waker: Option<Waker>,
}

/// The request future to be polled for asynchronous HTTP socket read operations.
pub(in crate) struct HttpRequestFuture {
    shared_http_request: Arc<Mutex<SharedHttpRequest>>,
}

impl HttpRequestFuture {
    /// Constructs a new `HttpRequestFuture`.
    fn new(port: u16) {
        let http_state = Arc::new(Mutex::new(SharedHttpRequest {
            state: HttpRequestFutureState::Uninitialized,
            string_data: String::new(),
            waker: None,
        }));;

    }
}

impl Future for HttpRequestFuture {
    type Output = Result<String, HttpRequestFutureError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = self.shared_http_request.lock().unwrap();

        match &state.state {
            // No HTTP request has been initialized yet.
            HttpRequestFutureState::Uninitialized => {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
            // The HTTP request operation has errored. Return a new HttpRequestFutureError.
            HttpRequestFutureState::Errored => {
                Poll::Ready(Err(HttpRequestFutureError { kind: HttpRequestFutureErrorKind::Unknown }))
            }
            // The HTTP request operation is in progress. Advance the future, reschedule, and return
            // a pending poll state.
            HttpRequestFutureState::InProgress => {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
            // The HTTP request operation has completed successfully. Return the raw HTTP request
            // string and complete the poll.
            HttpRequestFutureState::Completed => {
                Poll::Ready(Ok(state.string_data.clone()))
            }
        }
    }
}

