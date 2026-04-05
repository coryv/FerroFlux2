use sysinfo::System;

pub struct SystemResources {
    pub total_memory_bytes: u64,
    pub available_memory_bytes: u64,
    pub has_metal: bool,
    pub has_cuda: bool,
}

impl SystemResources {
    pub fn probe() -> Self {
        let mut sys = System::new_all();
        sys.refresh_memory();

        Self {
            total_memory_bytes: sys.total_memory(),
            available_memory_bytes: sys.available_memory(),
            has_metal: cfg!(feature = "metal"),
            has_cuda: cfg!(feature = "cuda"),
        }
    }

    pub fn available_gb(&self) -> f64 {
        self.available_memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }

    pub fn total_gb(&self) -> f64 {
        self.total_memory_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
    }
}
