use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct Timer {
    last_touch: Instant,
    time_processed: Duration,
    paused: bool,
    last_resume_time: Instant,
    last_pause_time: Instant,
    elapsed_time: Duration,
    elapsed_time_since_touch: Duration,
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            last_touch: Instant::now(),
            time_processed: Duration::ZERO,
            paused: true,
            last_resume_time: Instant::now(),
            last_pause_time: Instant::now(),
            elapsed_time: Duration::ZERO,
            elapsed_time_since_touch: Duration::ZERO,
        }
    }

    pub fn elapsed_time(&mut self) -> Duration {
        if !self.paused {
            self.elapsed_time = Instant::now().duration_since(self.last_resume_time);
        }
        self.elapsed_time
    }

    pub fn touch(&mut self) {
        self.last_touch = Instant::now();
        self.elapsed_time_since_touch = Duration::ZERO;
    }

    pub fn duration_since_touch(&mut self) -> Duration {
        self.elapsed_time_since_touch.saturating_add(match self.paused {
            true => Duration::ZERO,
            false => Instant::now().duration_since(self.last_touch.max(self.last_resume_time))
        })
    }

    pub fn process(&mut self, time: Duration) {
        self.time_processed = self.time_processed.saturating_add(time);
    }

    pub fn time_left_to_process(&mut self) -> Duration {
        if !self.paused {
            self.elapsed_time = Instant::now().duration_since(self.last_resume_time);
        }
        self.elapsed_time.saturating_sub(self.time_processed)
    }

    pub fn pause(&mut self) {
        if !self.paused {
            self.elapsed_time += Instant::now().duration_since(self.last_resume_time);
            self.elapsed_time_since_touch +=
                Instant::now().duration_since(self.last_touch.max(self.last_resume_time));
            self.last_pause_time = Instant::now();
            self.paused = true;
        }
    }

    pub fn resume(&mut self) {
        self.paused = false;
        self.last_resume_time = Instant::now();
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }
}
