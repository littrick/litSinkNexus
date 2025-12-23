use std::fmt::Display;
use tracing::log;
use windows::Win32::{Foundation::*, Globalization::*};
use windows::core::*;

pub fn init_i18n() {
    let langs = user_preferred_languages();

    log::debug!("User preferred UI languages: {:?}", langs);

    if let Some(lang) = langs.first() {
        rust_i18n::set_locale(lang);
        log::debug!("Set locale to {}", lang);
    }
}

fn user_preferred_languages() -> Vec<String> {
    let mut language = [0u16; 128];
    let mut pcc = language.len() as u32;
    let mut number = 0u32;

    unsafe {
        GetUserPreferredUILanguages(
            MUI_LANGUAGE_NAME,
            &mut number,
            Some(PWSTR::from_raw(language.as_mut_ptr())),
            &mut pcc,
        )
    }
    .unwrap();

    // Split the language array, which is null-terminated, into `number` PCWSTR strings
    let languages = language[..pcc as usize]
        .split(|&c| c == 0)
        .take(number as usize)
        .map(|lang| unsafe { PCWSTR::from_raw(lang.as_ptr()).to_string() }.unwrap())
        .collect();

    languages
}

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
        log::warn!("{}: {:?}", msg, self);
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

impl<T, E> WarnExt for std::result::Result<T, E>
where
    E: std::fmt::Debug,
{
    fn warn<C>(self, msg: C)
    where
        C: Display + Send + Sync + 'static,
    {
        if let Err(e) = self {
            log::warn!("{}: {:?}", msg, e);
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
