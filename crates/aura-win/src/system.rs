//! Disk space and hardware probe (ASR model recommendation, 006).

use crate::to_wide;
use serde::Serialize;
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, DXGI_ADAPTER_DESC1, DXGI_ADAPTER_FLAG_SOFTWARE, IDXGIFactory1,
};
use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows::core::PCWSTR;

/// Free bytes available to the user on the volume holding `path`.
pub fn free_space(path: &std::path::Path) -> Option<u64> {
    let wide = to_wide(&path.to_string_lossy());
    let mut free = 0u64;
    unsafe { GetDiskFreeSpaceExW(PCWSTR(wide.as_ptr()), Some(&mut free), None, None) }.ok()?;
    Some(free)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Gpu {
    pub name: String,
    pub vendor_id: u32,
    pub dedicated_vram_mb: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareInfo {
    pub ram_mb: u64,
    pub cpu_threads: u32,
    pub avx2: bool,
    pub gpus: Vec<Gpu>,
}

pub fn ram_mb() -> u64 {
    let mut m = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    unsafe { GlobalMemoryStatusEx(&mut m) }
        .map(|_| m.ullTotalPhys / (1024 * 1024))
        .unwrap_or(0)
}

pub fn gpus() -> Vec<Gpu> {
    let mut out = Vec::new();
    let Ok(factory) = (unsafe { CreateDXGIFactory1::<IDXGIFactory1>() }) else {
        return out;
    };
    let mut i = 0;
    while let Ok(adapter) = unsafe { factory.EnumAdapters1(i) } {
        i += 1;
        let Ok(desc): Result<DXGI_ADAPTER_DESC1, _> = (unsafe { adapter.GetDesc1() }) else {
            continue;
        };
        if desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32 != 0 {
            continue;
        }
        out.push(Gpu {
            name: crate::from_wide(&desc.Description),
            vendor_id: desc.VendorId,
            dedicated_vram_mb: desc.DedicatedVideoMemory as u64 / (1024 * 1024),
        });
    }
    out
}

pub fn probe() -> HardwareInfo {
    HardwareInfo {
        ram_mb: ram_mb(),
        cpu_threads: std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(1),
        avx2: avx2(),
        gpus: gpus(),
    }
}

#[cfg(target_arch = "x86_64")]
fn avx2() -> bool {
    std::arch::is_x86_feature_detected!("avx2")
}

#[cfg(not(target_arch = "x86_64"))]
fn avx2() -> bool {
    false
}
