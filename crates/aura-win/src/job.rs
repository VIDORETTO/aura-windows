//! Child processes (Codex app-server, aura-worker) are adopted into a Job
//! Object with `KILL_ON_JOB_CLOSE`, so they die with Aura even if it crashes
//! (OT-004 of 001, TK-001.3).

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
    SetInformationJobObject,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_SET_QUOTA, PROCESS_TERMINATE};

pub struct ChildJob {
    handle: HANDLE,
}

// SAFETY: a job handle can be used from any thread.
unsafe impl Send for ChildJob {}
unsafe impl Sync for ChildJob {}

impl ChildJob {
    pub fn new() -> windows::core::Result<Self> {
        unsafe {
            let handle = CreateJobObjectW(None, None)?;
            let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            SetInformationJobObject(
                handle,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const core::ffi::c_void,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )?;
            Ok(Self { handle })
        }
    }

    /// Adopts a running process by PID. Returns false if it already exited.
    pub fn adopt(&self, pid: u32) -> bool {
        unsafe {
            let Ok(process) = OpenProcess(PROCESS_SET_QUOTA | PROCESS_TERMINATE, false, pid) else {
                return false;
            };
            let ok = AssignProcessToJobObject(self.handle, process).is_ok();
            let _ = CloseHandle(process);
            ok
        }
    }
}

impl Drop for ChildJob {
    fn drop(&mut self) {
        // Closing the last handle kills every adopted child.
        unsafe {
            let _ = CloseHandle(self.handle);
        }
    }
}
