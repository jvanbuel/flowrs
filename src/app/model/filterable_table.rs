use anyhow::Error;

use super::{filter::Filter, StatefulTable};

pub struct FilterableTable<T> {
    pub table: StatefulTable<T>,
    pub filter: Filter,
    pub state: Vec<T>,
    pub errors: Vec<Error>,
    pub ticks: u64,
}
