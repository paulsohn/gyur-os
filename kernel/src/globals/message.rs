use heapless::mpmc::MpMcQueue;

use crate::message::Message;

const MSG_QUEUE_SIZE: usize = 16;

/// The Kernel message queue.
pub static MSG_QUEUE: MpMcQueue<Message, MSG_QUEUE_SIZE> = MpMcQueue::new();