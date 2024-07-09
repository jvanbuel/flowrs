use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

use crossterm::event::{self, KeyEvent, MouseEvent};

use super::custom::CustomEvent;

pub struct EventGenerator {
    pub _tick_rate: Duration,
    pub rx_event: Receiver<CustomEvent<KeyEvent, MouseEvent>>,
    pub _tx_event: Sender<CustomEvent<KeyEvent, MouseEvent>>,
}

impl EventGenerator {
    pub fn new(tick_rate: u16) -> Self {
        let (tx_event, rx_event) = channel::<CustomEvent<KeyEvent, MouseEvent>>();

        let tick_rate = Duration::from_millis(tick_rate as u64);
        let tx_event_thread = tx_event.clone();
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));
                if let Ok(true) = event::poll(timeout) {
                    if let Ok(ev) = event::read() {
                        tx_event_thread.send(CustomEvent::from(ev)).unwrap();
                    }
                }
                if last_tick.elapsed() > tick_rate {
                    tx_event_thread.send(CustomEvent::Tick).unwrap();
                    last_tick = Instant::now();
                }
            }
        });

        Self {
            _tick_rate: tick_rate,
            rx_event,
            _tx_event: tx_event,
        }
    }

    pub fn next(&self) -> Result<CustomEvent<KeyEvent, MouseEvent>, std::sync::mpsc::RecvError> {
        self.rx_event.recv()
    }
}
