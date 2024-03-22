use std::collections::{hash_map, HashMap};
use std::marker::PhantomData;

use compact_str::CompactString;
use exchange_algo::Orderbook;
use exchange_core::Exchange;

use self::sealed::State;
use crate::executor::TokioExecutor;
use crate::{ConnectorError, Executor, PairError, Read, Request, RuntimeError};

/// The Exchange runtime.
///
/// The runtime handles the connectors and plugs them into the specified
/// exchange.
pub struct Runtime<E, S: State> {
    pair: CompactString,
    orderbook: Orderbook,
    connector: HashMap<
        String,
        Box<
            dyn Read<Request = Request<<Orderbook as Exchange>::IncomingOrder>>
                + Send,
        >,
    >,
    processed: usize,
    executor: E,
    _state: PhantomData<S>,
}

impl Runtime<TokioExecutor, Idle> {
    #[inline]
    pub fn new(pair: &str, orderbook: Orderbook) -> Self {
        Runtime {
            pair: CompactString::new_inline(pair),
            orderbook,
            connector: Default::default(),
            processed: 0,
            executor: TokioExecutor,
            _state: PhantomData,
        }
    }
}

impl<E, S: State> Runtime<E, S> {
    #[inline]
    pub const fn orderbook(&self) -> &Orderbook {
        &self.orderbook
    }

    #[inline]
    pub const fn processed(&self) -> usize {
        self.processed
    }
}

impl<E> Runtime<E, Idle> {
    #[inline]
    pub fn with_connector(
        mut self,
        connector_id: impl Into<String>,
        connector: impl Read<Request = Request<<Orderbook as Exchange>::IncomingOrder>>
            + Send
            + 'static,
    ) -> Result<Self, RuntimeError> {
        let hash_map::Entry::Vacant(entry) =
            self.connector.entry(connector_id.into())
        else {
            return Err(ConnectorError::Duplicated)?;
        };

        entry.insert(Box::new(connector));

        Ok(self)
    }

    #[inline]
    pub async fn run(mut self) -> Result<Runtime<E, Dead>, RuntimeError>
    where
        E: Executor,
    {
        let (tx, rx) = flume::bounded(8192);

        for (connector_name, mut connector) in self.connector.drain() {
            let tx = tx.clone();
            self.executor.execute(async move {
                while let Ok(request) = connector.recv().await {
                    if let Err(err) = tx.send_async(request).await {
                        eprintln!("connector `{connector_name}` failed: {err}");
                        break;
                    }
                }
            });
        }

        drop(tx);

        while let Ok(request) = rx.recv_async().await {
            self.process(request)?;
        }

        let runtime = Runtime {
            pair: self.pair,
            orderbook: self.orderbook,
            connector: self.connector,
            processed: self.processed,
            executor: self.executor,
            _state: PhantomData,
        };

        Ok(runtime)
    }

    #[inline]
    fn process(
        &mut self,
        request: Request<<Orderbook as Exchange>::IncomingOrder>,
    ) -> Result<(), RuntimeError> {
        match request {
            Request::Create { ref pair, order } => {
                if pair != &self.pair {
                    return Err(PairError::Mismatch {
                        expected: self.pair.clone(),
                        found: pair.clone(),
                    })?;
                }

                let _ = self.orderbook.matching(order);
            }
            Request::Delete { order_id } => {
                self.orderbook.remove(&order_id);
            }
        };

        self.processed += 1;

        Ok(())
    }
}

pub enum Idle {}
pub enum Dead {}

mod sealed {
    pub trait State {}
    impl State for super::Dead {}
    impl State for super::Idle {}
}
