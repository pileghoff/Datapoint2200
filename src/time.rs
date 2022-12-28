use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct Instant {
    pub time: f64,
}

fn convert(ms: f64) -> u128 {
    (ms * 1000000.0) as u128
}

impl Instant {
    #[cfg(target_arch = "wasm32")]
    pub fn now() -> Instant {
        use web_sys::Performance;
        let window = web_sys::window().unwrap();
        let performance = window
            .performance()
            .expect("performance should be available");
        Instant {
            time: performance.now(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn now() -> Instant {
        let now = (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as f64)
            / 1_000_000.0;

        Instant { time: now }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn elapsed(&self) -> u128 {
        let now = (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as f64)
            / 1_000_000.0;

        convert(self.time - now)
    }

    #[cfg(target_arch = "wasm32")]
    pub fn elapsed(&self) -> u128 {
        use web_sys::Performance;
        let window = web_sys::window().unwrap();
        let performance = window
            .performance()
            .expect("performance should be available");
        convert(self.time - performance.now())
    }
}
