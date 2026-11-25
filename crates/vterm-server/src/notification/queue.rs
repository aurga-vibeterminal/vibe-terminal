use std::collections::VecDeque;
use std::time::{Duration, Instant};

use super::types::{NotificationError, NotificationRequest};

#[derive(Debug, Clone)]
pub(crate) struct QueuedNotification {
    pub request: NotificationRequest,
    pub attempt: u8,
    pub deliver_after: Instant,
}

#[derive(Debug)]
pub(crate) struct NotificationQueue {
    capacity: usize,
    items: VecDeque<QueuedNotification>,
}

impl NotificationQueue {
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            capacity,
            items: VecDeque::new(),
        }
    }

    pub(crate) fn enqueue(
        &mut self,
        request: NotificationRequest,
    ) -> Result<(), NotificationError> {
        self.enqueue_with_delay(request, 0, Duration::ZERO)
    }

    pub(crate) fn enqueue_with_delay(
        &mut self,
        request: NotificationRequest,
        attempt: u8,
        delay: Duration,
    ) -> Result<(), NotificationError> {
        if self.items.len() >= self.capacity {
            return Err(NotificationError::QueueFull);
        }

        self.items.push_back(QueuedNotification {
            request,
            attempt,
            deliver_after: Instant::now() + delay,
        });
        Ok(())
    }

    pub(crate) fn dequeue_due(&mut self) -> Option<QueuedNotification> {
        let now = Instant::now();
        let index = self
            .items
            .iter()
            .position(|entry| entry.deliver_after <= now)?;
        self.items.remove(index)
    }

    pub(crate) fn next_deadline(&self) -> Option<Instant> {
        self.items.iter().map(|entry| entry.deliver_after).min()
    }

    pub(crate) fn len(&self) -> usize {
        self.items.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::config::NotificationTargetConfig;

    use super::*;
    use crate::notification::types::NotificationPayload;

    #[test]
    fn queue_rejects_entries_when_full() {
        let mut queue = NotificationQueue::new(1);
        let request = NotificationRequest {
            event: "startup".to_string(),
            target: NotificationTargetConfig::default(),
            payload: NotificationPayload::new("title", "body"),
        };

        queue.enqueue(request.clone()).unwrap();
        let error = queue.enqueue(request).unwrap_err();

        assert!(matches!(error, NotificationError::QueueFull));
    }
}
