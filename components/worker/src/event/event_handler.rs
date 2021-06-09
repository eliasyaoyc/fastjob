use tokio::sync::mpsc::{Receiver, Sender};
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
    event_recv: Receiver<Event>,
    event_sender: Sender<Event>,
}

impl EventHandler {
    pub fn new(event_recv: Receiver<Event>,
               event_sender: Sender<Event>,
    ) -> Self {
        Self { event_recv, event_sender }
    }

    pub async fn process_event(&mut self) {
        while let Some(event) = self.event_recv.recv().await {

            // FAILURE_EVENT_QUEUE.try_with();

            match event {
                Event::AlarmEvent => {
                    info!("[EventHandler] receive a alarm event {}.");
                    match self.process_alarm_event().await {
                        Err(e) => {
                            error!("[EventHandler] process alarm event failed, error: {}", e);
                        }
                        _ => {}
                    }
                }
                Event::InstanceCompletedEvent(event) => {
                    info!("[EventHandler] receive a instance completed event {}.");
                    match self.process_instance_completed_event().await {
                        Err(e) => {
                            error!("[EventHandler] process instance completed event failed, error: {}", e);
                            FAILURE_EVENT_QUEUE.with(|failed| {});
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    pub async fn process_failed_event(&self) {}

    async fn process_alarm_event(&self) -> Result<()> {
        Ok(())
    }

    async fn process_instance_completed_event(&self) -> Result<()> {
        Ok(())
    }
}