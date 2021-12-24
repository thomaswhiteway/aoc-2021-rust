use std::time::{Duration, Instant};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::collections::HashMap;

#[derive(Default)]
struct Count {
    count: usize,
    duration: Duration,
}

impl Count {
    fn update(&mut self, duration: Duration) {
        self.count += 1;
        self.duration += duration;
    }

    fn nanos_per_op(&self) -> Option<u128> {
        if self.count > 0 {
            Some(self.duration.as_nanos() / self.count as u128)
        } else {
            None
        }
    }

    fn reset(&mut self) {
        self.count = 0;
        self.duration = Duration::from_millis(0);
    }
}

pub struct DurationTracker {
    operation: &'static str,
    start: Instant,
    tracker: Rc<Tracker>
}

impl Drop for DurationTracker {
    fn drop(&mut self) {
        self.tracker.report_duration(self.operation, self.start.elapsed())
    }
}

pub struct OperationTracker {
    tracker: Rc<Tracker>
}

impl OperationTracker {
    fn new(tracker: Rc<Tracker>) -> Self {
        OperationTracker {
            tracker
        }
    }
}
impl Drop for OperationTracker {
    fn drop(&mut self) {
        self.tracker.done()
    }
}

pub struct Tracker {
    report_interval: usize,
    count: Cell<usize>,
    durations: RefCell<HashMap<&'static str, Count>>,
}

impl Tracker {
    pub fn new(report_interval: usize) -> Rc<Self> {
        Rc::new(Tracker {
            report_interval,
            count: Cell::new(0),
            durations: RefCell::new(Default::default()),
        })
    }

    fn report_duration(&self, operation: &'static str, duration: Duration) {
        self.durations.borrow_mut().entry(operation).or_default().update(duration)
    }

    fn done(&self) {
        let count = self.count.get() + 1;
        self.count.set(count);

        if count % self.report_interval == 0 {
            {
                let durations = self.durations.borrow();
                print!("{}: ", count);
                for (index, (operation, duration_count)) in durations.iter().enumerate() {
                    if index > 0 {
                        print!(", ");
                    }
                    print!("{} {} (x{})",
                        operation,
                        duration_count.count,
                        duration_count.nanos_per_op().map(|val| format!("{}ns", val)).unwrap_or_else(|| "n/a".to_string()));
                }
                println!();
            }

            self.durations.borrow_mut().values_mut().for_each(|count| count.reset());
        }
    }
}

pub trait OperationTrack {
    type DurationTracker;
    fn track_duration(&self, operation: &'static str) -> Self::DurationTracker;
}

pub trait Track {
    type OperationTracker: OperationTrack;
    fn track_operation(&self) -> Self::OperationTracker;
}

impl Track for Rc<Tracker> {
    type OperationTracker = OperationTracker;

    fn track_operation(&self) -> Self::OperationTracker {
        OperationTracker::new(self.clone())

    }
}

impl OperationTrack for OperationTracker {
    type DurationTracker = DurationTracker;

    fn track_duration(&self, operation: &'static str) -> Self::DurationTracker {
        DurationTracker {
            operation,
            start: Instant::now(),
            tracker: self.tracker.clone()
        }
    }
}

impl Track for () {
    type OperationTracker = ();

    fn track_operation(&self) -> Self::OperationTracker {
        ()
    }
}

impl OperationTrack for () {
    type DurationTracker = ();

    fn track_duration(&self, _operation: &'static str) -> Self::DurationTracker {
        ()
    }
}
