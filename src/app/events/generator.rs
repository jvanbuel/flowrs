use std::time::Duration;

use crossterm::event::EventStream;
use futures::StreamExt;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::time::{interval, MissedTickBehavior};

use super::custom::FlowrsEvent;

pub struct EventGenerator {
    _tick_rate: Duration,
    pub rx_event: Receiver<FlowrsEvent>,
    _tx_event: Sender<FlowrsEvent>,
}

impl EventGenerator {
    pub fn new(tick_rate: u16) -> Self {
        let (tx_event, rx_event) = channel::<FlowrsEvent>(500);

        let tick_rate = Duration::from_millis(u64::from(tick_rate));
        let tx_event_thread = tx_event.clone();
        // `EventStream` reads terminal events asynchronously (crossterm does the
        // blocking read on its own thread), so this loop never blocks a tokio
        // worker. Ticks come from an independent timer running alongside it.
        tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut ticker = interval(tick_rate);
            // Don't fire a burst of catch-up ticks if the loop is ever delayed.
            ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
            loop {
                let event = tokio::select! {
                    _ = ticker.tick() => FlowrsEvent::Tick,
                    maybe_event = reader.next() => match maybe_event {
                        Some(Ok(ev)) => FlowrsEvent::from(ev),
                        // A read error (terminal unusable) or a closed source both
                        // mean there is nothing more to read, so stop the loop
                        // rather than spin retrying.
                        Some(Err(_)) | None => break,
                    },
                };
                // A send error means the receiver is gone (the app is exiting),
                // so stop the loop instead of spinning forever.
                if tx_event_thread.send(event).await.is_err() {
                    break;
                }
            }
        });

        Self {
            _tick_rate: tick_rate,
            rx_event,
            _tx_event: tx_event,
        }
    }

    pub async fn next(&mut self) -> Option<FlowrsEvent> {
        self.rx_event.recv().await
    }
}
