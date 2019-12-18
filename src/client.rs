extern crate indy_ledger_client;

use indy_ledger_client::config::pool_factory::PoolFactory;

fn main() {
  let factory = PoolFactory::from_genesis_file("genesis.txn").unwrap();
  print!("{:?}", factory.merkle_tree)
}
