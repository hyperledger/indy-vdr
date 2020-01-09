use std::thread;
use std::time::Duration;

extern crate env_logger;
extern crate indy_ledger_client;

use indy_ledger_client::config::{LedgerResult, PoolFactory};

fn test() -> LedgerResult<()> {
  let factory = PoolFactory::from_genesis_file("genesis.txn")?;
  let pool = factory.create_pool()?;
  thread::sleep(Duration::from_secs(10));
  Ok(())
}

fn main() -> LedgerResult<()> {
  env_logger::init();
  test()
}
