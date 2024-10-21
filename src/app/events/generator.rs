use std::time::{Duration, Instant};

use tokio::sync::mpsc::{channel, Receiver, Sender};

use crossterm::event;

use super::custom::FlowrsEvent;

pub struct EventGenerator {
    pub _tick_rate: Duration,
    pub rx_event: Receiver<FlowrsEvent>,
    pub tx_event: Sender<FlowrsEvent>,
}

impl EventGenerator {
    pub fn new(tick_rate: u16) -> Self {
        let (tx_event, rx_event) = channel::<FlowrsEvent>(500);

        let tick_rate = Duration::from_millis(tick_rate as u64);
        let tx_event_thread = tx_event.clone();
        tokio::spawn(async move {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));
                if let Ok(true) = event::poll(timeout) {
                    if let Ok(ev) = event::read() {
                        tx_event_thread.send(FlowrsEvent::from(ev)).await;
                    }
                }
                if last_tick.elapsed() > tick_rate {
                    tx_event_thread.send(FlowrsEvent::Tick).await;
                    last_tick = Instant::now();
                }
            }
        });

        Self {
            _tick_rate: tick_rate,
            rx_event,
            tx_event,
        }
    }

    pub async fn next(&mut self) -> Option<FlowrsEvent> {
        self.rx_event.recv().await
    }
}
