use std::time::Instant;

pub struct FrameCounter {
    last_frame_instant: Instant,
    last_report_instant: Instant,
    frames_since_last_report: u64,
    pending_report: Option<f64>,
    report_interval: f64,
}

impl FrameCounter {
    pub fn new(report_interval: f64) -> Self {
        Self {
            last_frame_instant: Instant::now(),
            last_report_instant: Instant::now(),
            frames_since_last_report: 0,
            pending_report: None,
            report_interval,
        }
    }

    pub fn new_frame(&mut self) -> f64 {
        let now = Instant::now();
        let elapsed_since_last_report = (now - self.last_report_instant).as_secs_f64();
        let frame_delta = (now - self.last_frame_instant).as_secs_f64();

        if elapsed_since_last_report >= self.report_interval {
            self.pending_report = Some(self.frames_since_last_report as f64 / elapsed_since_last_report);
            self.last_report_instant = now;
            self.frames_since_last_report = 0;
        }

        self.last_frame_instant = now;
        self.frames_since_last_report += 1;
        frame_delta
    }

    pub fn report(&mut self) -> Option<f64> {
        self.pending_report.take()
    }
}
