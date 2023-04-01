use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::Future;

// State of the future
// Polling - future is not yet resolved
// Done - future is resolved and contains the result
enum State<F, T, E>
where
    F: Future<Output = Result<T, E>>,
{
    Polling(F),
    Done(T),
}

// Structure holding the state of two futures
// Polling - both futures are not yet resolved
// Done - both futures are resolved
enum TryJoin<A, B, AR, BR, E>
where
    A: Future<Output = Result<AR, E>>,
    B: Future<Output = Result<BR, E>>,
{
    Polling {
        a: State<A, AR, E>,
        b: State<B, BR, E>,
    },
    Done,
}

// Our TryJoin structure behaves like a future
impl<A, B, AR, BR, E> Future for TryJoin<A, B, AR, BR, E>
where
    A: Future<Output = Result<AR, E>>,
    B: Future<Output = Result<BR, E>>,
{
    // We are returning a tuple of results of both futures
    type Output = Result<(AR, BR), E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Unsafe way of getting a mutable reference to self
        // We need to pinky swear that we won't move ourselves
        let this = unsafe { self.get_unchecked_mut() };

        // Check our state and get our inner futures
        // Nobody should be polling us if we are done
        let (a, b) = match this {
            Self::Polling { a, b } => (a, b),
            Self::Done => panic!("polled after completion"),
        };

        // If the state is Polling, it means it was not yet resolved
        // We will create a pin out of it so we can poll it and if it is ready
        // Switch the state to Done and store the result of the future there
        if let State::Polling(fut) = a {
            if let Poll::Ready(res) = unsafe { Pin::new_unchecked(fut) }.poll(cx) {
                *a = State::Done(res?);
            };
        }

        if let State::Polling(fut) = b {
            if let Poll::Ready(res) = unsafe { Pin::new_unchecked(fut) }.poll(cx) {
                *b = State::Done(res?);
            };
        }

        match (a, b) {
            // If both futures are resolved, we won't be polled again, so we can move ourselves despite the pin
            // We cannot be really in Done state already, so that arm is unreachable
            // We extract future values in the Polling state and return them as a tuple
            (State::Done(_), State::Done(_)) => match std::mem::replace(this, Self::Done) {
                Self::Polling {
                    a: State::Done(a),
                    b: State::Done(b),
                } => Ok((a, b)).into(),
                _ => unreachable!(),
            },
            // If any of the futures is not yet resolved, we return Pending and the polling will continue
            _ => Poll::Pending,
        }
    }
}

pub fn try_join<A, B, AR, BR, E>(a: A, b: B) -> impl Future<Output = Result<(AR, BR), E>>
where
    A: Future<Output = Result<AR, E>>,
    B: Future<Output = Result<BR, E>>,
{
    // Initially we start in a Polling state with both futures in the Future state
    TryJoin::Polling {
        a: State::Polling(a),
        b: State::Polling(b),
    }
}
