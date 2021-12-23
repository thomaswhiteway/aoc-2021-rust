use std::collections::{BinaryHeap, HashSet};
use std::hash::Hash;

pub trait State: Sized {
    fn min_remaining_cost(&self) -> usize;
    fn successors(&self) -> Box<dyn Iterator<Item = (Self, usize)> + '_>;
    fn is_complete(&self) -> bool;
}

pub fn solve<S: Eq + Hash + State + Clone>(initial_state: S) -> Option<usize> {
    let mut heap: BinaryHeap<Candidate<S>> = BinaryHeap::new();
    let mut visited: HashSet<S> = HashSet::new();

    heap.push(Candidate::new(initial_state, 0));

    while let Some(candidate) = heap.pop() {
        if candidate.state.is_complete() {
            return Some(candidate.cost);
        }

        if visited.contains(&candidate.state) {
            continue;
        }

        visited.insert(candidate.state.clone());

        for next_candidate in candidate.successors() {
            if !visited.contains(&next_candidate.state) {
                heap.push(next_candidate);
            }
        }
    }

    None
}

#[derive(PartialEq, Eq, Debug)]
struct Candidate<S> {
    state: S,
    cost: usize,
    min_remaining_cost: usize,
}

impl<S: State> Candidate<S> {
    fn new(state: S, cost: usize) -> Self {
        let min_remaining_cost = state.min_remaining_cost();
        Candidate {
            state,
            cost,
            min_remaining_cost,
        }
    }

    fn successors(&self) -> impl Iterator<Item = Candidate<S>> + '_ {
        self.state
            .successors()
            .map(|(state, cost)| Self::new(state, self.cost + cost))
    }
}

impl<S: PartialEq> PartialOrd for Candidate<S> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            (self.cost + self.min_remaining_cost)
                .cmp(&(other.cost + other.min_remaining_cost))
                .reverse(),
        )
    }
}

impl<S: Eq> Ord for Candidate<S> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}
