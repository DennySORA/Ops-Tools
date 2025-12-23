use indicatif::{ProgressBar, ProgressStyle};

/// 進度追蹤器
pub struct Progress {
    bar: ProgressBar,
}

impl Progress {
    /// 建立進度條
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

    /// 增加 1
    pub fn inc(&self) {
        self.bar.inc(1);
    }

    /// 增加指定數量
    #[allow(dead_code)]
    pub fn inc_by(&self, delta: u64) {
        self.bar.inc(delta);
    }

    /// 更新訊息
    #[allow(dead_code)]
    pub fn set_message(&self, message: &str) {
        self.bar.set_message(message.to_string());
    }

    /// 完成（保留進度條）
    #[allow(dead_code)]
    pub fn finish(&self) {
        self.bar.finish();
    }

    /// 完成並顯示訊息
    pub fn finish_with_message(&self, message: &str) {
        self.bar.finish_with_message(message.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_creation() {
        let progress = Progress::new(100, "測試");
        progress.inc();
        progress.finish();
    }
}
