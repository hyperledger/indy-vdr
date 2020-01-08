extern crate indy_ledger_client;

// use indy_ledger_client::config::pool_factory::PoolFactory;

use zmq;

fn main() {
  // let factory = PoolFactory::from_genesis_file("genesis.txn").unwrap();
  // print!("{:?}", factory.merkle_tree)

  let zmq_ctx = zmq::Context::new();
  let send_cmd_sock = zmq_ctx.socket(zmq::SocketType::PAIR).unwrap();
  let recv_cmd_sock = zmq_ctx.socket(zmq::SocketType::PAIR).unwrap();

  let inproc_sock_name: String = format!("inproc://{}", addr);
  recv_cmd_sock.bind(inproc_sock_name.as_str()).unwrap();
  send_cmd_sock.connect(inproc_sock_name.as_str()).unwrap();
}

fn work(socket: zmq::Socket) {
  loop {
    let item = socket.as_poll_item(zmq::POLLIN);
    let poll_items = [item];
    let poll_res = zmq::poll(&mut poll_items, 0);
    if poll_res == 0 {
      self.events.push_back(PoolEvent::Timeout(req_id, alias)); // TODO check duplicate ?
    }
  }
}
