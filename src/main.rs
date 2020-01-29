
use std::net::TcpListener;

use reverseProxy::ThreadPool;
mod handle_connection;
use handle_connection::route;



fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(1024);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection::route(stream);
        });
    }
}

