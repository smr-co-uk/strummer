// Copyright 2026 smr.co.uk ltd
// SPDX-License-Identifier: Apache-2.0

use std::{fmt, io, path::PathBuf};

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    Usage(String),
    Io { path: PathBuf, source: io::Error },
    Parse(String),
    Validation(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage(message) | Self::Parse(message) | Self::Validation(message) => {
                write!(f, "{message}")
            }
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
        }
    }
}

impl std::error::Error for AppError {}
