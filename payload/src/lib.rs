use std::net::TcpStream;

#[ctor::ctor]
fn ctor() {
    println!("Hi From Lib!");
    TcpStream::connect("127.0.0.1:7788").unwrap();
}
