//! UAC onayıyla yönetici olarak süreç çalıştırma (ShellExecuteExW "runas") ve çıkış kodunu bekleme.
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum ElevateError {
    #[error("kullanıcı yönetici iznini reddetti veya başlatılamadı")]
    Denied,
}

#[cfg(windows)]
pub fn run_as_admin(exe: &Path, args: &str) -> Result<u32, ElevateError> {
    use std::os::windows::ffi::OsStrExt;
    use windows::core::{w, PCWSTR};
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{GetExitCodeProcess, WaitForSingleObject, INFINITE};
    use windows::Win32::UI::Shell::{ShellExecuteExW, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW};
    use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;

    let file: Vec<u16> = exe
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let params: Vec<u16> = args.encode_utf16().chain(std::iter::once(0)).collect();
    let mut info = SHELLEXECUTEINFOW {
        cbSize: u32::try_from(std::mem::size_of::<SHELLEXECUTEINFOW>()).unwrap_or(0),
        fMask: SEE_MASK_NOCLOSEPROCESS,
        lpVerb: w!("runas"),
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: PCWSTR(params.as_ptr()),
        nShow: SW_HIDE.0,
        ..Default::default()
    };
    // SAFETY: arabellekler çağrı boyunca yaşar; dönen süreç tanıtıcısı aşağıda kapatılır.
    unsafe {
        ShellExecuteExW(&mut info).map_err(|_| ElevateError::Denied)?;
        if info.hProcess.is_invalid() {
            return Err(ElevateError::Denied);
        }
        WaitForSingleObject(info.hProcess, INFINITE);
        let mut code = 1u32;
        let _ = GetExitCodeProcess(info.hProcess, &mut code);
        let _ = CloseHandle(info.hProcess);
        Ok(code)
    }
}

#[cfg(not(windows))]
pub fn run_as_admin(_exe: &Path, _args: &str) -> Result<u32, ElevateError> {
    Err(ElevateError::Denied)
}
