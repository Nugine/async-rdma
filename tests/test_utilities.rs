use async_rdma::{Rdma, RdmaListener};
use futures::Future;
use portpicker::pick_unused_port;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::{io, net::ToSocketAddrs};

const PORT_NUM: u8 = 1;
const GID_INDEX: usize = 1;
const MAX_MSG_LEN: usize = 128;

type RdmaFn<R> = fn(Rdma) -> R;

#[tokio::main]
async fn server_wrapper<A: ToSocketAddrs, R: Future<Output = Result<(), io::Error>>>(
    addr: A,
    f: RdmaFn<R>,
) -> io::Result<()> {
    let rdma = RdmaListener::bind(addr)
        .await?
        .accept(PORT_NUM, GID_INDEX, MAX_MSG_LEN)
        .await?;
    f(rdma).await
}

#[tokio::main]
async fn client_wrapper<A: ToSocketAddrs, R: Future<Output = Result<(), io::Error>>>(
    addr: A,
    f: RdmaFn<R>,
) -> io::Result<()> {
    let rdma = Rdma::connect(addr, PORT_NUM, GID_INDEX, MAX_MSG_LEN).await?;
    f(rdma).await
}

/// Used to make two-end testing easy
#[allow(unused)] // It's actually used in other tests
pub(crate) fn test_server_client<
    SR: Future<Output = Result<(), io::Error>> + 'static,
    CR: Future<Output = Result<(), io::Error>> + 'static,
>(
    s: RdmaFn<SR>,
    c: RdmaFn<CR>,
) {
    let addr = get_unused_ipv4_addr();
    let server = std::thread::spawn(move || server_wrapper(addr, s));
    std::thread::sleep(std::time::Duration::from_secs(1));
    let client = std::thread::spawn(move || client_wrapper(addr, c));
    let _ = client.join().unwrap();
    let _ = server.join().unwrap();
}

pub(crate) fn get_unused_ipv4_addr() -> SocketAddrV4 {
    SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), pick_unused_port().unwrap())
}
