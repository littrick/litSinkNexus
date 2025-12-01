use std::fmt::Display;

use windows::Win32::Foundation::HWND;
use windows::core::{BOOL, HRESULT};
use windows::core::{Error, Result};

pub fn win_error<C, T>(msg: C) -> anyhow::Result<T>
where
    C: Display + Send + Sync + 'static,
{
    anyhow::bail!(anyhow::Error::from(Error::from_thread()).context(msg));
}

pub fn win_warn<C>(msg: C)
where
    C: Display + Send + Sync + 'static,
{
    Error::from_thread().warn(msg);
}

pub trait WarnExt {
    fn warn<C>(self, msg: C)
    where
        C: Display + Send + Sync + 'static;
}

impl WarnExt for BOOL {
    fn warn<C>(self, msg: C)
    where
        C: Display + Send + Sync + 'static,
    {
        if !self.as_bool() {
            win_warn(msg);
        }
    }
}

impl WarnExt for Error {
    fn warn<C>(self, msg: C)
    where
        C: Display + Send + Sync + 'static,
    {
        tracing::log::warn!("{}: {:?}", msg, self);
    }
}

impl WarnExt for HRESULT {
    fn warn<C>(self, msg: C)
    where
        C: Display + Send + Sync + 'static,
    {
        if self.is_err() {
            Error::from_hresult(self).warn(msg);
        }
    }
}

impl<T> WarnExt for Result<T> {
    fn warn<C>(self, msg: C)
    where
        C: Display + Send + Sync + 'static,
    {
        if let Err(e) = self {
            tracing::log::warn!("{}: {:?}", msg, e);
        }
    }
}

impl<T> WarnExt for anyhow::Result<T> {
    fn warn<C>(self, msg: C)
    where
        C: Display + Send + Sync + 'static,
    {
        if let Err(e) = self {
            tracing::log::warn!("{}: {:?}", msg, e);
        }
    }
}

pub trait WinBoolExt {
    fn context<C>(self, msg: C) -> anyhow::Result<()>
    where
        C: Display + Send + Sync + 'static;
}

impl WinBoolExt for BOOL {
    fn context<C>(self, msg: C) -> anyhow::Result<()>
    where
        C: Display + Send + Sync + 'static,
    {
        if self.as_bool() {
            Ok(())
        } else {
            win_error(msg)
        }
    }
}

pub trait ToWinResult<T> {
    fn to_win_result(self) -> Result<T>;
}

impl<T> ToWinResult<T> for anyhow::Result<T> {
    fn to_win_result(self) -> Result<T> {
        self.map_err(|e| Error::new(Error::from_thread().code(), e.to_string()))
    }
}

/// 为HWND实现Send和Sync
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WndHandle(HWND);

impl WndHandle {
    pub fn new(hwnd: HWND) -> Self {
        Self(hwnd)
    }

    pub fn hwnd(&self) -> HWND {
        self.0
    }
}
unsafe impl Send for WndHandle {}
unsafe impl Sync for WndHandle {}
