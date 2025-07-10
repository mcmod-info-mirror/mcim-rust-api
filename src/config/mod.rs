pub mod _redis;
pub mod database;

use redis::aio::MultiplexedConnection;
use std::sync::Arc;