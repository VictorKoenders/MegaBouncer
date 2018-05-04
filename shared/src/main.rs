extern crate mio_extras;
extern crate shared;

use mio_extras::channel::*;
use shared::mio::*;
use std::io::{Read, Result, Error, ErrorKind, Write};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread::spawn;

fn main() {
    let mut process = Command::new("../target/debug/test_tool")
        .arg("sleep")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn_async()
        .expect("Could not spawn process");
    let poll = Poll::new().expect("Could not spawn poll");
    let mut events = Events::with_capacity(10);
    let token = Token(1);
    process.register(&poll, token, Ready::all(), PollOpt::edge()).expect("Could not register");
    'outer: loop {
        poll.poll(&mut events, None).expect("Could not poll");
        for event in &events {
            assert_eq!(event.token(), token);
            let result = process.try_recv();
            println!("{:?}", result);

            match result {
                Ok(ProcessEvent::Exit(_)) => break 'outer,
                Ok(ProcessEvent::IoError(_, e)) | Ok(ProcessEvent::CommandError(e)) => {
                    if e.kind() != ErrorKind::WouldBlock  {
                        break 'outer;
                    }
                },
                Err(_) => break 'outer,
                _ => {}
            }
        }
    }
}

pub trait CommandAsync {
    fn spawn_async(&mut self) -> Result<Process>;
}

impl CommandAsync for Command {
    fn spawn_async(&mut self) -> Result<Process> {
        let child = self.spawn()?;
        Ok(Process::from_child(child))
    }
}

pub struct Process {
    receiver: Receiver<ProcessEvent>,
    stdin: Option<std::process::ChildStdin>,
}

impl Process {
    pub fn try_recv(&mut self) -> std::result::Result<ProcessEvent, std::sync::mpsc::TryRecvError> {
        self.receiver.try_recv()
    }

    pub fn write(&mut self, data: &[u8]) -> Result<usize> {
        match self.stdin.as_mut() {
            Some(ref mut stdin) => stdin.write(data),
            None => Err(Error::from(ErrorKind::NotConnected)),
        }
    }
}

impl Evented for Process {
    fn register(&self, poll: &Poll, token: Token, interest: Ready, ops: PollOpt) -> Result<()> {
        self.receiver.register(poll, token, interest, ops)
    }
    fn reregister(&self, poll: &Poll, token: Token, interest: Ready, ops: PollOpt) -> Result<()> {
        self.receiver.reregister(poll, token, interest, ops)
    }
    fn deregister(&self, poll: &Poll) -> Result<()> {
        self.receiver.deregister(poll)
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum ShouldAbort {
    Yes,
    No,
}

fn try_send_buffer(
    buffer: &[u8],
    channel: StdioChannel,
    sender: &Sender<ProcessEvent>,
) -> ShouldAbort {
    let str = match std::str::from_utf8(buffer) {
        Ok(s) => s,
        Err(e) => {
            let _ = sender.send(ProcessEvent::Utf8Error(channel, e));
            return ShouldAbort::Yes;
        }
    };
    if str.is_empty() {
        println!("Aborting try_send_buffer because we're sending empty strings");
        println!("Channel: {:?}", channel);
        return ShouldAbort::Yes;
    }
    if let Err(_) = sender.send(ProcessEvent::Data(channel, String::from(str))) {
        ShouldAbort::Yes
    } else {
        ShouldAbort::No
    }
}

fn create_reader<T: Read + 'static>(
    mut stream: T,
    sender: Sender<ProcessEvent>,
    channel: StdioChannel,
) -> impl FnOnce() {
    move || {
        let mut buffer = [0u8; 256];
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => {
                    // if we read 0 bytes from the stream, that means the stream ended
                    break;
                }
                Ok(n) => {
                    if ShouldAbort::Yes == try_send_buffer(&buffer[..n], channel, &sender) {
                        break;
                    }
                }
                Err(e) => {
                    let _ = sender.send(ProcessEvent::IoError(channel, e));
                    break;
                }
            }
        }
    }
}

impl Process {
    pub fn from_child(mut child: Child) -> Process {
        let (sender, receiver) = channel();
        if let Some(stdout) = child.stdout.take() {
            spawn(create_reader(stdout, sender.clone(), StdioChannel::Stdout));
        }
        if let Some(stderr) = child.stderr.take() {
            spawn(create_reader(stderr, sender.clone(), StdioChannel::Stderr));
        }
        let stdin = child.stdin.take();
        spawn(move || {
            let result = match child.wait_with_output() {
                Err(e) => {
                    let _ = sender.send(ProcessEvent::CommandError(e));
                    return;
                }
                Ok(r) => r,
            };
            if !result.stdout.is_empty() {
                if ShouldAbort::Yes == try_send_buffer(&result.stdout, StdioChannel::Stdout, &sender) {
                    return;
                }
            }
            if !result.stderr.is_empty() {
                if ShouldAbort::Yes == try_send_buffer(&result.stderr, StdioChannel::Stderr, &sender) {
                    return;
                }
            }
            let _ = sender.send(ProcessEvent::Exit(result.status));
        });
        Process { receiver, stdin }
    }
}

#[derive(Debug)]
pub enum ProcessEvent {
    Data(StdioChannel, String),
    CommandError(std::io::Error),
    IoError(StdioChannel, std::io::Error),
    Utf8Error(StdioChannel, std::str::Utf8Error),
    Exit(ExitStatus),
}

#[derive(Copy, Clone, Debug)]
pub enum StdioChannel {
    Stdout,
    Stderr,
    Other,
}

/*
pub fn main(){
    let mut client = shared::client::Client::new("Test", ());
    client.register_listener("*", catch_all);
    client.launch();
}

fn catch_all(_: &mut (), name: &str, value: &shared::serde_json::Value) {
    println!("{:?} {:?}", name, value);
}
*/
