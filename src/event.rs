use std::time::{Duration, Instant};
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};

#[derive(Debug, Clone)]
pub enum Event { Tick, Key(KeyEvent), Resize(u16, u16) }

pub struct EventLoop { tick_rate: Duration, last_tick: Instant }
impl EventLoop {
    pub fn new(tick_rate: Duration) -> Self { Self { tick_rate, last_tick: Instant::now() } }
    pub fn next(&mut self) -> anyhow::Result<Event> {
        let timeout = self.tick_rate.saturating_sub(self.last_tick.elapsed());
        if event::poll(timeout)? {
            match event::read()? {
                CrosstermEvent::Key(k) => return Ok(Event::Key(k)),
                CrosstermEvent::Resize(w,h) => return Ok(Event::Resize(w,h)),
                _ => {}
            }
        }
        if self.last_tick.elapsed() >= self.tick_rate { self.last_tick = Instant::now(); Ok(Event::Tick) } else { Ok(Event::Tick) }
    }
}
