use crate::core::Epoch;
use crate::slot_clock::{Slot, SlotClock as SlotClockT};
use async_trait::async_trait;
use libp2p::futures::future::BoxFuture;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::time::Duration;
use tokio::sync::mpsc::error::TrySendError;
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{error, trace, warn};

/// Trait implemented for exposing a ticking mechanism.
///
/// Given that the expected usage of this trait is in a looped [tokio::select!],
/// it is expected that the implementation is cancellation safe, as per
/// the [tokio::select!] docs:
///
/// Cancellation safety can be defined in the following way: If you have
/// a future that has not yet completed, then it must be a no-op to drop
/// that future and recreate it.
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
#[async_trait]
pub trait Ticker<Item>: Send + Sync + 'static
where
    Item: Send + Sync + 'static,
{
    /// Waits for the tick to trigger and returns [Some(Item)].
    ///
    /// Returns [None] if the ticker is stopped.
    async fn tick(&mut self) -> Option<Item>;
}

/// Builder for tickers based on [SlotClock].
pub struct SlotClockTickerBuilder<SlotClock> {
    _marker: PhantomData<SlotClock>,
}

impl<SlotClock> SlotClockTickerBuilder<SlotClock>
where
    SlotClock: SlotClockT + 'static,
{
    pub fn new() -> Self {
        Self {
            _marker: Default::default(),
        }
    }

    pub fn build_epoch_ticker(
        &self,
        slot_clock: SlotClock,
        slots_per_epoch: u64,
    ) -> (EpochTicker, BoxFuture<'static, ()>) {
        let (tx, rx) = tokio::sync::mpsc::channel(1);

        let future = Box::pin(async move {
            loop {
                if let Err(e) = tick_fn(
                    slot_clock.duration_to_next_epoch(slots_per_epoch),
                    &tx,
                    || slot_clock.now().map(|s| s.epoch(slots_per_epoch)),
                )
                .await
                {
                    error!(error = ?e, "Epoch ticker channel is closed, stopping");
                    return;
                }
            }
        });

        (rx.into(), future)
    }

    pub fn build_slot_ticker(&self, slot_clock: SlotClock) -> (SlotTicker, BoxFuture<'static, ()>) {
        let (tx, rx) = tokio::sync::mpsc::channel(1);

        let future = Box::pin(async move {
            loop {
                if let Err(e) =
                    tick_fn(slot_clock.duration_to_next_slot(), &tx, || slot_clock.now()).await
                {
                    error!(error = ?e, "Slot ticker channel is closed, stopping");
                    return;
                }
            }
        });

        (rx.into(), future)
    }
}

/// Function used for sending one tick on the provided [Sender], after an optional delay.
///
/// If the channel is full, the tick [Item] is dropped. Errors out if the channel is closed.
async fn tick_fn<Item, F>(
    sleep_duration: Option<Duration>,
    tx: &Sender<Item>,
    item_fn: F,
) -> Result<(), String>
where
    Item: Debug + Copy,
    F: Fn() -> Option<Item>,
{
    if let Some(duration) = sleep_duration {
        tokio::time::sleep(duration).await;
    }

    match item_fn() {
        Some(item) => match tx.try_send(item) {
            Ok(_) => trace!(item = ?item, "Item tick"),
            Err(e) => match e {
                TrySendError::Full(_) => warn!(item = ?item, "Item channel full, dropping item"),
                TrySendError::Closed(_) => return Err("item channel is closed".to_string()),
            },
        },
        None => {
            warn!("Item not available");
        }
    }

    Ok(())
}

/// [Epoch] ticker that wraps around [Receiver] to ensure cancellation safety.
pub struct EpochTicker(Receiver<Epoch>);

impl From<Receiver<Epoch>> for EpochTicker {
    fn from(value: Receiver<Epoch>) -> Self {
        Self(value)
    }
}

#[async_trait]
impl Ticker<Epoch> for EpochTicker {
    async fn tick(&mut self) -> Option<Epoch> {
        self.0.recv().await
    }
}

/// [Slot] ticker that wraps around [Receiver] to ensure cancellation safety.
pub struct SlotTicker(Receiver<Slot>);

impl From<Receiver<Slot>> for SlotTicker {
    fn from(value: Receiver<Slot>) -> Self {
        Self(value)
    }
}

#[async_trait]
impl Ticker<Slot> for SlotTicker {
    async fn tick(&mut self) -> Option<Slot> {
        self.0.recv().await
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod epoch_ticker {
        use super::*;
        use crate::slot_clock::MockSlotClock;

        #[tokio::test]
        async fn tick() {
            let slots_per_epoch = 10;

            let mut slot_clock_mock = MockSlotClock::default();
            slot_clock_mock
                .expect_duration_to_next_epoch()
                .returning(move |s| {
                    assert_eq!(s, slots_per_epoch);

                    Some(Duration::from_millis(1))
                });

            let mut slot_count = 0;

            slot_clock_mock.expect_now().returning(move || {
                slot_count += 10;

                Some(Slot::new(slot_count))
            });

            let (mut ticker, ticker_fut) =
                SlotClockTickerBuilder::new().build_epoch_ticker(slot_clock_mock, slots_per_epoch);

            tokio::spawn(ticker_fut);

            assert_eq!(ticker.tick().await, Some(Epoch::new(1)));
            assert_eq!(ticker.tick().await, Some(Epoch::new(2)));
            assert_eq!(ticker.tick().await, Some(Epoch::new(3)));
        }

        #[tokio::test]
        async fn no_tick() {
            let slot_clock_mock = MockSlotClock::default();

            let (mut ticker, _) =
                SlotClockTickerBuilder::new().build_epoch_ticker(slot_clock_mock, 10);

            assert_eq!(ticker.tick().await, None);
        }
    }

    mod slot_ticker {
        use super::*;
        use crate::slot_clock::MockSlotClock;

        #[tokio::test]
        async fn tick() {
            let mut slot_clock_mock = MockSlotClock::default();
            slot_clock_mock
                .expect_duration_to_next_slot()
                .returning(|| Some(Duration::from_millis(1)));

            let mut slot_count = 0;

            slot_clock_mock.expect_now().returning(move || {
                slot_count += 1;

                Some(Slot::new(slot_count))
            });

            let (mut ticker, ticker_fut) =
                SlotClockTickerBuilder::new().build_slot_ticker(slot_clock_mock);

            tokio::spawn(ticker_fut);

            assert_eq!(ticker.tick().await, Some(Slot::new(1)));
            assert_eq!(ticker.tick().await, Some(Slot::new(2)));
            assert_eq!(ticker.tick().await, Some(Slot::new(3)));
        }

        #[tokio::test]
        async fn no_tick() {
            let slot_clock_mock = MockSlotClock::default();

            let (mut ticker, _) = SlotClockTickerBuilder::new().build_slot_ticker(slot_clock_mock);

            assert_eq!(ticker.tick().await, None);
        }
    }

    mod tick_fn {
        use super::*;
        use tokio::sync::mpsc::error::TryRecvError;

        #[tokio::test]
        async fn no_timeout() {
            let (tx, mut rx) = tokio::sync::mpsc::channel(1);
            let item_fn = || Some(());

            tick_fn(None, &tx, item_fn).await.unwrap();
            rx.recv().await.unwrap();
        }

        #[tokio::test]
        async fn timeout() {
            let (tx, mut rx) = tokio::sync::mpsc::channel(1);
            let item_fn = || Some(());

            tick_fn(Some(Duration::from_millis(10)), &tx, item_fn)
                .await
                .unwrap();
            rx.recv().await.unwrap();
        }

        #[tokio::test]
        async fn item_channel_full() {
            let (tx, mut rx) = tokio::sync::mpsc::channel(1);
            let item_fn = || Some(());

            tick_fn(None, &tx, item_fn).await.unwrap();
            tick_fn(None, &tx, item_fn).await.unwrap();

            rx.recv().await.unwrap();

            let res = rx.try_recv();
            assert!(res.is_err());
            assert!(matches!(res.unwrap_err(), TryRecvError::Empty));
        }

        #[tokio::test]
        async fn item_channel_closed() {
            let (tx, _) = tokio::sync::mpsc::channel(1);
            let item_fn = || Some(());

            assert!(tick_fn(None, &tx, item_fn).await.is_err());
        }

        #[tokio::test]
        async fn item_not_available() {
            let (tx, mut rx) = tokio::sync::mpsc::channel(1);
            let item_fn = || None::<()>;

            tick_fn(None, &tx, item_fn).await.unwrap();

            let res = rx.try_recv();
            assert!(res.is_err());
            assert!(matches!(res.unwrap_err(), TryRecvError::Empty));
        }
    }
}
