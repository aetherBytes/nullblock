use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::Stream;
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::events::ArbEvent;
use crate::server::AppState;

pub async fn scanner_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.subscribe_events();
    let stream = BroadcastStream::new(rx);

    let event_stream = stream.filter_map(|result| {
        match result {
            Ok(event) => {
                // Filter for scanner-related events
                if event.topic.starts_with("arb.scanner.") {
                    let data = serde_json::to_string(&event).unwrap_or_default();
                    Some(Ok(Event::default().event("signal").data(data)))
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    });

    Sse::new(event_stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

pub async fn edges_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.subscribe_events();
    let stream = BroadcastStream::new(rx);

    let event_stream = stream.filter_map(|result| {
        match result {
            Ok(event) => {
                // Filter for edge-related events
                if event.topic.starts_with("arb.edge.") {
                    let data = serde_json::to_string(&event).unwrap_or_default();
                    Some(Ok(Event::default().event("edge").data(data)))
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    });

    Sse::new(event_stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

pub async fn all_events_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.subscribe_events();
    let stream = BroadcastStream::new(rx);

    let event_stream = stream.filter_map(|result| match result {
        Ok(event) => {
            let data = serde_json::to_string(&event).unwrap_or_default();
            Some(Ok(Event::default().data(data)))
        }
        Err(_) => None,
    });

    Sse::new(event_stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

pub async fn threat_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.subscribe_events();
    let stream = BroadcastStream::new(rx);

    let event_stream = stream.filter_map(|result| match result {
        Ok(event) => {
            if event.topic.starts_with("arb.threat.") {
                let data = serde_json::to_string(&event).unwrap_or_default();
                Some(Ok(Event::default().event("threat").data(data)))
            } else {
                None
            }
        }
        Err(_) => None,
    });

    Sse::new(event_stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

pub async fn helius_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.subscribe_events();
    let stream = BroadcastStream::new(rx);

    let event_stream = stream.filter_map(|result| match result {
        Ok(event) => {
            if event.topic.starts_with("arb.helius.") {
                let event_name = event.topic.split('.').last().unwrap_or("helius");
                let data = serde_json::to_string(&event).unwrap_or_default();
                Some(Ok(Event::default().event(event_name).data(data)))
            } else {
                None
            }
        }
        Err(_) => None,
    });

    Sse::new(event_stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}

pub async fn positions_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.subscribe_events();
    let stream = BroadcastStream::new(rx);

    let event_stream = stream.filter_map(|result| match result {
        Ok(event) => {
            if event.topic.starts_with("arb.position.") {
                let event_name = event.topic.split('.').last().unwrap_or("position");
                let data = serde_json::to_string(&event).unwrap_or_default();
                Some(Ok(Event::default().event(event_name).data(data)))
            } else {
                None
            }
        }
        Err(_) => None,
    });

    Sse::new(event_stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}
