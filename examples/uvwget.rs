extern crate libuv;
use curl::multi::{Multi, Socket, SocketEvents};
use libuv::prelude::*;
use libuv::{AsyncHandle, PollHandle, PollEvents, TimerHandle};
use std::cell::Cell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

enum Message {
    Timeout(u64),
    Socket(Socket, SocketEvents, usize),
}

struct MessageQueue {
    notify: AsyncHandle,
    queue: Arc<Mutex<VecDeque<Message>>>,
}

impl MessageQueue {
    fn new<F>(r#loop: &Loop, mut cb: F) -> Result<MessageQueue, libuv::Error>
    where
        F: FnMut(&mut VecDeque<Message>) + 'static
    {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let queue_clone = queue.clone();
        let notify = r#loop.r#async(move |_| {
            if let Ok(mut q) = queue_clone.lock() {
                cb(&mut q);
            }
        })?;
        Ok(MessageQueue { notify, queue })
    }

    fn send(&mut self, msg: Message) -> Result<(), Box<(dyn std::error::Error + '_)>> {
        let mut q = self.queue.lock()?;
        q.push_back(msg);
        self.notify.send()?;
        Ok(())
    }
}

fn check_multi_info(curl_handle: &Rc<Multi>) {
    curl_handle.messages(|msg| {
        // TODO
    });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let urls: Vec<String> = std::env::args().skip(1).collect();
    if urls.len() == 0 {
        eprintln!("Usage: uvwget [url1] [url2] ...");
        return Ok(())
    }

    let mut r#loop = Loop::default()?;

    let mut timeout = r#loop.timer()?;
    let mut poll_handles: Vec<PollHandle> = Vec::new();
    let mut curl_handle = Rc::new(Multi::new());
    let mut curl_handle2 = curl_handle.clone();
    let mut queue = Rc::new(MessageQueue::new(&r#loop, move |messages| {
        for msg in messages.drain(..) {
            match msg {
                Message::Timeout(timeout_ms) => {
                    if let Err(e) = timeout.start(|_| {
                        curl_handle2.timeout();
                    }, timeout_ms, 0) {
                        eprintln!("Could not start timer: {}", e);
                    }
                },
                Message::Socket(socket, action, token) => {
                    if action.input() || action.output() {
                        let poll = if token > 0 {
                            unsafe { poll_handles.get_unchecked_mut(token - 1) }
                        } else {
                            match r#loop.poll_socket(socket) {
                                Ok(p) => {
                                    let idx = poll_handles.len();
                                    poll_handles.push(p);
                                    curl_handle.assign(socket, idx + 1);
                                    unsafe { poll_handles.get_unchecked_mut(idx) }
                                },
                                Err(e) => {
                                    eprintln!("Could not open poll: {}", e);
                                    return;
                                }
                            }
                        };

                        let _ = timeout.stop();
                        if action.input() {
                            poll.start(PollEvents::READABLE, curl_perform);
                        } else {
                            poll.start(PollEvents::WRITABLE, curl_perform);
                        }
                    } else if action.remove() {
                        if token > 0 {
                            let poll = unsafe { poll_handles.get_unchecked_mut(token - 1) };
                            poll.stop();
                            curl_handle.assign(socket, 0);
                        }
                    }
                },
            }
        }
    })?);

    {
        let queue = queue.clone();
        curl_handle.socket_function(move |socket, action, token| {
            if let Err(e) = queue.send(Message::Socket(socket, action, token)) {
                eprintln!("Could not queue socket: {}", e);
            }
        })?;
    }

    curl_handle.timer_function(move |duration| {
        let mut timeout_ms = match duration {
            Some(x) => x.as_millis() as u64,
            None => 1,
        };
        if timeout_ms == 0 {
            timeout_ms = 1;
        }
        match queue.send(Message::Timeout(timeout_ms)) {
            Ok(_) => true,
            Err(e) => {
                eprintln!("Could not queue timeout: {}", e);
                false
            }
        }
    })?;

    r#loop.run(RunMode::Default)?;

    Ok(())
}
