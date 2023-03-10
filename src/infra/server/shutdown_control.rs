use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct ShutDownControl {
    status: Arc<AtomicU8>,
    enable: Arc<AtomicBool>,
}

#[derive(Debug, Clone, Default)]
pub struct ShutDown {
    status: Arc<AtomicU8>,   //关闭进度
    enable: Arc<AtomicBool>, //是否需要等待通知
}

impl ShutDown {
    pub fn new(status: Arc<AtomicU8>, enable: Arc<AtomicBool>) -> ShutDown {
        ShutDown { status, enable }
    }
    pub async fn wait_close(&self) {
        while self.status.load(Ordering::Relaxed) < 2 {
            sleep(Duration::from_millis(10)).await;
        }
    }
    //给关闭设置一个超时时间
    #[allow(dead_code)]
    pub async fn close_timeout(&self, timeout: Duration) {
        if !self.enable.load(Ordering::Relaxed) {
            return;
        }
        self.status.fetch_add(1, Ordering::Relaxed);
        let _ = tokio::time::timeout(timeout, async move {
            while self.status.load(Ordering::Relaxed) == 1 {
                sleep(Duration::from_millis(10)).await;
            }
        });
        self.status.fetch_add(1, Ordering::Relaxed);
    }
    //等待关闭 直到跳出为止
    pub async fn close(&self) {
        if !self.enable.load(Ordering::Relaxed) {
            return;
        }
        self.status.fetch_add(1, Ordering::Relaxed);
        while self.status.load(Ordering::Relaxed) == 1 {
            sleep(Duration::from_millis(10)).await;
        }
    }
}

impl ShutDownControl {
    pub fn new() -> ShutDownControl {
        Self {
            status: Arc::new(AtomicU8::new(0)),
            enable: Arc::new(AtomicBool::new(false)),
        }
    }
    pub fn generate_shutdown(&self) -> ShutDown {
        ShutDown::new(self.status.clone(), self.enable.clone())
    }
    //等待接受停止信号
    pub async fn wait(&self) {
        self.enable.store(true, Ordering::Relaxed);
        while self.status.load(Ordering::Relaxed) == 0 {
            sleep(Duration::from_millis(10)).await;
        }
    }
    //收尾动作完成后响应
    pub async fn down(&self) {
        self.status.store(2, Ordering::Relaxed);
    }
}

impl From<ShutDown> for ShutDownControl {
    fn from(value: ShutDown) -> Self {
        ShutDownControl {
            status: value.status,
            enable: value.enable,
        }
    }
}
