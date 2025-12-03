use indicatif::{ProgressBar, ProgressStyle};

pub struct ProgressTracker {
    bar: ProgressBar,
}

impl ProgressTracker {
    pub fn new(total: u64, message: &str) -> Self {
        let bar = ProgressBar::new(total);
        bar.set_style(
            ProgressStyle::default_bar()
                .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")
                .expect("Failed to create progress style")
                .progress_chars("=>-"),
        );
        bar.set_message(message.to_string());

        Self { bar }
    }

    pub fn new_spinner(message: &str) -> Self {
        let bar = ProgressBar::new_spinner();
        bar.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .expect("Failed to create spinner style"),
        );
        bar.set_message(message.to_string());

        Self { bar }
    }

    pub fn inc(&self) {
        self.bar.inc(1);
    }

    pub fn inc_by(&self, delta: u64) {
        self.bar.inc(delta);
    }

    pub fn set_position(&self, pos: u64) {
        self.bar.set_position(pos);
    }

    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }

    pub fn finish(&self) {
        self.bar.finish();
    }

    pub fn finish_with_message(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }

    pub fn finish_and_clear(&self) {
        self.bar.finish_and_clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_tracker_creation() {
        let tracker = ProgressTracker::new(100, "測試");
        tracker.finish();
    }

    #[test]
    fn test_progress_tracker_spinner() {
        let tracker = ProgressTracker::new_spinner("載入中");
        tracker.finish();
    }
}
