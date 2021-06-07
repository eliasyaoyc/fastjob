use tokio::sync::mpsc::Receiver;
use crate::error::Result;
use dashmap::DashMap;
use fastjob_components_utils::event::Event;
use std::cell::RefCell;
use std::collections::HashMap;

const EVENT_MAX_RETRY_TIMES: usize = 5;

thread_local! {
    static FAILURE_EVENT_QUEUE: RefCell<HashMap<usize,Vec<Event>>> = RefCell::new(HashMap::with_capacity(1024));
}

pub struct EventHandler {
    event_stream: Receiver<Event>,
}

impl EventHandler {
    pub fn new(event_stream: Receiver<Event>) -> Self {
        Self { event_stream }
    }

    pub async fn process_event(&mut self) {
        while let Some(event) = self.event_stream.recv().await {

            // FAILURE_EVENT_QUEUE.try_with();

            match event {
                Event::AlarmEvent => {
                    info!("[EventHandler] receive a alarm event {}.");
                    match self.process_alarm_event().await {
                        Err(e) => {}
                        _ => {}
                    }
                }
                Event::InstanceCompletedEvent => {
                    info!("[EventHandler] receive a instance completed event {}.")
                    match self.process_instance_completed_event().await {
                        Err(e) => {}
                        _ => {}
                    }
                }
            }
        }
    }

    async fn process_alarm_event(&self) -> Result<()> {
        Ok(())
    }

    async fn process_instance_completed_event(&self) -> Result<()> {
        Ok(())
    }
}