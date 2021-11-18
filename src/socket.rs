use super::types::Action;
use std::{
    io::{self, Error, ErrorKind, Read, Write},
    net::{TcpStream, ToSocketAddrs},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, MutexGuard,
    },
    thread,
};

/// Small Abstraction to keep the code cleaner
pub struct Socket {
    stream: TcpStream,
    actions: Arc<Mutex<Vec<Action>>>,
    should_drop: Arc<AtomicBool>,
}

impl Drop for Socket {
    fn drop(&mut self) {
        self.should_drop.store(true, Ordering::SeqCst);
    }
}

impl Socket {
    pub fn new(addr: impl ToSocketAddrs) -> io::Result<Self> {
        let stream = TcpStream::connect(addr)?;

        Ok(Self {
            stream,
            actions: Arc::new(Mutex::new(vec![])),
            should_drop: Arc::new(AtomicBool::new(false)),
        })
    }

    fn reader_loop(
        mut stream: TcpStream,
        actions: &Mutex<Vec<Action>>,
        should_drop: &AtomicBool,
    ) -> io::Result<()> {
        // If Mutex couldn't be locked, actions will be backed up here and get added the next time
        // we acquire the Mutex lock.
        // Note: Not 100% sure that this is needed, just made sense when I thought about it.
        // I think that it makes sense to keep receieving data and store it, then when the Mutex
        // lock is acquired, we can add the already stored data + new data to it, thus we'll
        // probably block the rendering thread a lot less.
        // This might cause stuff like a user typing something and instead of each character
        // that they type lighting up individually, 2 characters might light up at the same time,
        // so I might just ditch this method entirely later.
        let mut buffer = [0u8; 5];
        let mut actions_backup: Vec<Action> = vec![];

        while !should_drop.load(Ordering::SeqCst) {
            if stream.read(&mut buffer)? < 1 {
                break;
            }

            let action = match &buffer[..4] {
                b"Join" => Action::Join(buffer[4]),
                b"Left" => Action::Left(buffer[4].into()),
                _ => Action::Input((buffer[0].into(), buffer[1].into())),
            };

            if let Ok(mut vec) = actions.try_lock() {
                vec.extend(actions_backup.drain(..).as_slice());
                vec.push(action);
                continue;
            }

            actions_backup.push(action);
        }

        Ok(())
    }

    pub fn init_reader(&self) -> io::Result<()> {
        let actions = self.actions.clone();
        let should_drop = self.should_drop.clone();
        let stream = self.stream.try_clone()?;
        thread::spawn(move || Self::reader_loop(stream, &actions, &should_drop));
        Ok(())
    }

    pub fn send_input(&mut self, input: char) -> io::Result<()> {
        self.stream.write(&[input as u8])?;
        Ok(())
    }

    pub fn create_session(&mut self) -> io::Result<u16> {
        self.stream.write("Create".as_bytes())?;

        let mut buffer = [0u8; 2];

        let size = self.stream.read(&mut buffer)?;

        if size < 2 {
            return Err(Error::new(
                ErrorKind::OutOfMemory,
                "Session Couldn't be created.",
            ));
        }

        Ok(u16::from_be_bytes(buffer))
    }

    /// It'll join an already existing session and return the position of the player.
    pub fn join_session(&mut self, session_token: impl Into<u16>) -> io::Result<u8> {
        let be_bytes = session_token.into().to_be_bytes();

        let buffer = [b'J', b'o', b'i', b'n', be_bytes[0], be_bytes[1]];

        self.stream.write(&buffer)?;

        let mut buffer = [0u8; 1];

        if self.stream.read(&mut buffer)? < 1 {
            return Err(Error::new(
                ErrorKind::ConnectionAborted,
                "Couldn't Join Session.",
            ));
        }

        Ok(buffer[0])
    }

    pub fn actions(&self) -> MutexGuard<Vec<Action>> {
        self.actions.lock().unwrap()
    }
}
