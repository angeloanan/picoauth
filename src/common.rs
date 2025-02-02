use std::sync::LazyLock;

use regex::Regex;

pub static USERNAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_]{3,32}$").unwrap());
