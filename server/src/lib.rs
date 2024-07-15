mod error;
mod threadpool;
use common::Message;
use error::*;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod, SslStream};
use std::{
    fmt::Debug, io::{stdin, BufRead, BufReader, BufWriter, Write}, net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream}, sync::{mpsc::{self, Receiver, Sender}, Arc, Mutex, Weak}, thread::{self, JoinHandle}
};
use threadpool::ThreadPool;

const MAX_CONNECTIONS: usize = 2;
const IP: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const PORT: u16 = 1337;
const CERT: &str = "cert.pem";
const KEY: &str = "key.pem";

pub fn run() -> Result<()> {
    let socket = SocketAddrV4::new(IP, PORT);
    let listener = TcpListener::bind(socket)?;
    let acceptor = ssl_acceptor();

    let _handle = thread::spawn(move || listen(listener, acceptor));

    start_cli();

    Ok(())
}

fn start_cli() {
    for cmd in stdin().lines() {
        let cmd = match cmd {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Not recognized: {}", e);
                continue;
            }
        };
        println!("Got command: {}", cmd);
    }
}

fn ssl_acceptor() -> Arc<SslAcceptor> {
    let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    acceptor
        .set_private_key_file(KEY, SslFiletype::PEM)
        .unwrap();
    acceptor
        .set_certificate_file(CERT, SslFiletype::PEM)
        .unwrap();
    acceptor.check_private_key().unwrap();
    Arc::new(acceptor.build())
}

fn listen(listener: TcpListener, acceptor: Arc<SslAcceptor>) -> Result<()> {
    let thread_pool = ThreadPool::new(MAX_CONNECTIONS)?;
    let registry = Registry::new();

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => {
                let acceptor = acceptor.clone();
                let peer_addr = s.peer_addr().unwrap();
                match acceptor.accept(s) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Invalid SSL connection from: {}\nError: {}", peer_addr, e);
                        continue;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error occurred: {}", e);
                continue;
            }
        };
        let registry = registry.clone();

        thread_pool.execute(move || {
            if let Err(e) = handle_connection(&mut stream, registry) {
                eprintln!("Error: {}", e)
            }
        })?;
    }
    Ok(())
}

fn handle_connection(stream: &mut SslStream<TcpStream>, registry: Arc<Mutex<Registry>>) -> Result<()> {
    let mut buffer = BufReader::new(stream);
    /* let recv = BufWriter::new(stream);
    let requests = buffer.lines();

    let mut user_builder = UserBuilder::new();

    let (tx, rx) = mpsc::channel();
    thread::spawn(move|| for update in rx{
        recv.write(update);
    });
     */
    let mut user_builder = UserBuilder::new();

    let auth_request = get_request(&mut buffer)?;

    dbg!(&auth_request);

    if let Request::CONNECT(user, pass) = auth_request{
        println!("connected user: {}", user);
        user_builder.set_username(user);
        user_builder.authenticate(pass)?;
    } else {
        return Err(ServerError::AuthenticationFailed)
    }

    let user = Arc::new(user_builder.build());

    // THIS:

    registry.lock().unwrap().add_user(Arc::downgrade(&user));

    /* OR THIS: (?)
    {
        let reg = registry.lock().unwrap();
        reg.add_user(user);
    }
    */

    while let Ok(request) = get_request(&mut buffer){
        let mut registry = registry.lock().unwrap();

        match request {
            Request::CONNECT(user, pass) => {
                // ALREADY CONNECTED!
            },
            Request::SEND(loc, content) => {
                let username = user.get_username();
                registry.add_message(Message::new(username.to_string(), content.clone()));
                println!("got some message from {}: {}", username, content);
            }
            _ => ()
        }
    }

    Ok(())
}

#[allow(dead_code)]
#[derive(Debug)]
enum Request {
    // From client
    CONNECT(String, String),
    FETCH(/* channels | messages */), // -> Result(data, reason)
    SEND(String, String),              // -> Result(timestamp, reason)
    SEARCH,
}

fn get_request<B: BufRead + Debug>(buffer: &mut B) -> Result<Request>{
    let mut request = Vec::new();

    buffer.read_until(b'\t', &mut request)?;

    let mut lines = request.lines();

    let header = lines.next().unwrap()?;
    let location = lines.next().unwrap()?;
    let mut body = String::new();
    for line in lines{
        let mut line = line?;
        line.push('\n');
        body.push_str(&line);
    }

    match header.as_str(){
        "CONNECT" => {
            Ok(Request::CONNECT(location, body))
        },
        "SEND" => Ok(Request::SEND("none".to_string(), body)),
        _ => panic!()
    }
}

pub struct Registry{
    received_messages: Vec<Message>,
    connected_users: Vec<Weak<User>>
}

impl Registry{
    pub fn new() -> Arc<Mutex<Self>>{
        Arc::new(Mutex::new(Self { received_messages: vec!(), connected_users: vec!()}))
    }

    pub fn add_message(&mut self, message: Message){
        self.notify_all(&message);
        self.received_messages.push(message);
    }

    pub fn add_user(&mut self, user: Weak<User>){
        self.connected_users.push(user)
    }

    fn notify_all(&self, message: &Message){
        for user in &self.connected_users {
            // TODO: HANDLE THIS UNWRAP
            //(*user.upgrade().unwrap()).notify(&message);
        }
    }
}

pub struct UserBuilder{
    username: Option<String>
}

impl UserBuilder{
    pub fn new() -> Self{
        Self {username: None}
    }

    pub fn set_username(&mut self, username: String){
        self.username = Some(username)
    }

    pub fn authenticate(&mut self, password: String) -> Result<()>{
        if let None = self.username{
            return Err(ServerError::AuthenticationFailed)
        }

        // TODO: Actual authentication.

        Ok(())
    }

    pub fn build(self) -> User{
        User { username: self.username.unwrap() }
    }
}

pub struct User{
    username: String,
    //transmitter: Sender<Vec<u8>>
}

impl User{
    pub fn get_username(&self) -> &str{
        &self.username
    }
    /* pub fn notify(&self, message: &Message){
        self.transmitter.send(message.as_bytes());
    } */
}