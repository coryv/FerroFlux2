use anyhow::Result;
use candle_core::{DType, Device};

/// Select the best available device according to priority: Metal -> CUDA -> CPU.
pub fn select_device() -> Result<Device> {
    #[cfg(feature = "metal")]
    {
        if let Ok(device) = Device::new_metal(0) {
            tracing::info!("Using Metal backend");
            return Ok(device);
        }
    }

    #[cfg(feature = "cuda")]
    {
        if let Ok(device) = Device::new_cuda(0) {
            tracing::info!("Using CUDA backend");
            return Ok(device);
        }
    }

    tracing::warn!("No GPU backend available, falling back to CPU");
    Ok(Device::Cpu)
}

/// Select the preferred dtype based on the device.
/// On GPU (Metal/CUDA), use BF16. On CPU, use F32.
pub fn select_dtype(device: &Device) -> DType {
    if device.is_cuda() || device.is_metal() {
        DType::BF16
    } else {
        DType::F32
    }
}
