use std::error::Error;
use std::time::Duration;

use tokio::sync::broadcast;

pub trait ConnectionMessage: Clone + Send + 'static {
    type Frame;
    fn connected() -> Self;
    fn disconnected() -> Self;
    fn frame(frame: Self::Frame) -> Self;
}

pub const INITIAL_BACKOFF: Duration = Duration::from_secs(1);
pub const MAX_BACKOFF: Duration = Duration::from_secs(60);

pub fn run_blocking_actor<M, A, I, S>(
    attempts: A,
    tx: broadcast::Sender<M>,
    mut sleeper: S,
) where
    M: ConnectionMessage,
    A: IntoIterator<Item = Result<I, Box<dyn Error>>>,
    I: IntoIterator<Item = M::Frame>,
    S: FnMut(Duration),
{
    let mut backoff = INITIAL_BACKOFF;
    for attempt in attempts {
        match attempt {
            Err(e) => {
                log::warn!("connect failed: {e:?}; retrying in {backoff:?}");
                let _ = tx.send(M::disconnected());
                sleeper(backoff);
                backoff = (backoff * 2).min(MAX_BACKOFF);
            }
            Ok(iter) => {
                let _ = tx.send(M::connected());
                let mut produced = false;
                for frame in iter {
                    produced = true;
                    if tx.send(M::frame(frame)).is_err() {
                        return;
                    }
                }
                let _ = tx.send(M::disconnected());
                if produced {
                    log::warn!("iterator ended after producing data; reconnecting");
                    backoff = INITIAL_BACKOFF;
                } else {
                    log::warn!("iterator ended without producing data; retrying in {backoff:?}");
                    sleeper(backoff);
                    backoff = (backoff * 2).min(MAX_BACKOFF);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {
        Connected,
        Disconnected,
        Frame(u32),
    }

    impl ConnectionMessage for TestMsg {
        type Frame = u32;
        fn connected() -> Self {
            TestMsg::Connected
        }
        fn disconnected() -> Self {
            TestMsg::Disconnected
        }
        fn frame(f: u32) -> Self {
            TestMsg::Frame(f)
        }
    }

    fn drain(rx: &mut broadcast::Receiver<TestMsg>) -> Vec<TestMsg> {
        let mut out = Vec::new();
        while let Ok(m) = rx.try_recv() {
            out.push(m);
        }
        out
    }

    fn err(msg: &'static str) -> Box<dyn Error> {
        msg.into()
    }

    #[test]
    fn success_emits_connected_frames_disconnected_no_sleep() {
        let (tx, mut rx) = broadcast::channel(64);
        let attempts: Vec<Result<Vec<u32>, Box<dyn Error>>> = vec![Ok(vec![1, 2, 3])];
        let sleeps = Arc::new(Mutex::new(Vec::<Duration>::new()));
        let sleeps_c = Arc::clone(&sleeps);

        run_blocking_actor::<TestMsg, _, _, _>(
            attempts,
            tx,
            |d| sleeps_c.lock().unwrap().push(d),
        );

        assert_eq!(
            drain(&mut rx),
            vec![
                TestMsg::Connected,
                TestMsg::Frame(1),
                TestMsg::Frame(2),
                TestMsg::Frame(3),
                TestMsg::Disconnected,
            ]
        );
        assert!(
            sleeps.lock().unwrap().is_empty(),
            "no sleep expected on healthy success path"
        );
    }

    #[test]
    fn build_failure_emits_disconnected_and_sleeps_initial_backoff() {
        let (tx, mut rx) = broadcast::channel(64);
        let attempts: Vec<Result<Vec<u32>, Box<dyn Error>>> = vec![Err(err("boom"))];
        let sleeps = Arc::new(Mutex::new(Vec::<Duration>::new()));
        let sleeps_c = Arc::clone(&sleeps);

        run_blocking_actor::<TestMsg, _, _, _>(
            attempts,
            tx,
            |d| sleeps_c.lock().unwrap().push(d),
        );

        assert_eq!(drain(&mut rx), vec![TestMsg::Disconnected]);
        assert_eq!(*sleeps.lock().unwrap(), vec![INITIAL_BACKOFF]);
    }

    #[test]
    fn consecutive_failures_backoff_exponentially_capped_at_max() {
        let (tx, _rx) = broadcast::channel(256);
        let attempts: Vec<Result<Vec<u32>, Box<dyn Error>>> =
            (0..10).map(|_| Err(err("boom"))).collect();
        let sleeps = Arc::new(Mutex::new(Vec::<Duration>::new()));
        let sleeps_c = Arc::clone(&sleeps);

        run_blocking_actor::<TestMsg, _, _, _>(
            attempts,
            tx,
            |d| sleeps_c.lock().unwrap().push(d),
        );

        let expected = vec![
            Duration::from_secs(1),
            Duration::from_secs(2),
            Duration::from_secs(4),
            Duration::from_secs(8),
            Duration::from_secs(16),
            Duration::from_secs(32),
            Duration::from_secs(60),
            Duration::from_secs(60),
            Duration::from_secs(60),
            Duration::from_secs(60),
        ];
        assert_eq!(*sleeps.lock().unwrap(), expected);
    }

    #[test]
    fn successful_frame_resets_backoff() {
        let (tx, _rx) = broadcast::channel(64);
        let attempts: Vec<Result<Vec<u32>, Box<dyn Error>>> = vec![
            Err(err("boom1")),
            Err(err("boom2")),
            Ok(vec![42]),
            Err(err("boom3")),
        ];
        let sleeps = Arc::new(Mutex::new(Vec::<Duration>::new()));
        let sleeps_c = Arc::clone(&sleeps);

        run_blocking_actor::<TestMsg, _, _, _>(
            attempts,
            tx,
            |d| sleeps_c.lock().unwrap().push(d),
        );

        assert_eq!(
            *sleeps.lock().unwrap(),
            vec![
                Duration::from_secs(1), // after first failure
                Duration::from_secs(2), // after second failure (exp backoff)
                Duration::from_secs(1), // after third failure: backoff reset by yielded frame
            ]
        );
    }

    #[test]
    fn empty_iter_sleeps_with_backoff_to_avoid_busy_loop() {
        let (tx, _rx) = broadcast::channel(64);
        let attempts: Vec<Result<Vec<u32>, Box<dyn Error>>> = vec![Ok(vec![]), Ok(vec![])];
        let sleeps = Arc::new(Mutex::new(Vec::<Duration>::new()));
        let sleeps_c = Arc::clone(&sleeps);

        run_blocking_actor::<TestMsg, _, _, _>(
            attempts,
            tx,
            |d| sleeps_c.lock().unwrap().push(d),
        );

        assert_eq!(
            *sleeps.lock().unwrap(),
            vec![Duration::from_secs(1), Duration::from_secs(2)]
        );
    }

    #[test]
    fn exits_when_subscribers_dropped_mid_iteration() {
        let (tx, rx) = broadcast::channel::<TestMsg>(64);
        drop(rx);
        let attempts_called = Arc::new(Mutex::new(0u32));
        let attempts_called_c = Arc::clone(&attempts_called);
        let attempts: Vec<Result<Vec<u32>, Box<dyn Error>>> =
            vec![Ok(vec![1, 2, 3]), Ok(vec![4, 5, 6])];
        let counted = attempts.into_iter().inspect(move |_| {
            *attempts_called_c.lock().unwrap() += 1;
        });

        run_blocking_actor::<TestMsg, _, _, _>(counted, tx, |_| panic!("no sleep expected"));

        assert_eq!(
            *attempts_called.lock().unwrap(),
            1,
            "loop must abort after first send-frame failure, not continue to second attempt"
        );
    }
}
