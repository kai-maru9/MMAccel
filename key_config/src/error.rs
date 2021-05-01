#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{}が見つかりませんでした", .0)]
    FileNotFound(String),
    #[error("IOエラー: {}", .0)]
    Io(std::io::Error),
    #[error("エラー {}: (0x{:08x}){}", .1, .0.code().0, .0.message())]
    HResult(windows::Error, String),
    #[error("{}({}:{})にエラーがあります", .1, .0.line(), .0.column())]
    JsonFile(serde_json::Error, String),
    #[error("データがおかしいです")]
    InvalidData,
    #[error("ウィンドウを作成できませんでした ({})", .0)]
    Wita(wita::ApiError),
}

impl Error {
    pub fn file(e: std::io::Error, path: impl AsRef<std::path::Path>) -> Self {
        if e.kind() == std::io::ErrorKind::NotFound {
            Self::FileNotFound(path.as_ref().to_string_lossy().to_string())
        } else {
            Self::Io(e)
        }
    }

    pub fn hresult(e: windows::Error, text: impl AsRef<str>) -> Self {
        Self::HResult(e, text.as_ref().to_string())
    }

    pub fn json_file(e: serde_json::Error, path: impl AsRef<std::path::Path>) -> Self {
        Self::JsonFile(e, path.as_ref().to_string_lossy().to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(src: std::io::Error) -> Error {
        Error::Io(src)
    }
}

impl From<wita::ApiError> for Error {
    fn from(src: wita::ApiError) -> Error {
        Error::Wita(src)
    }
}
