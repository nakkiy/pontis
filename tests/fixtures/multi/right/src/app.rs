pub fn title() -> &'static str {
    "pontis-next"
}

pub fn feature_flags() -> Vec<&'static str> {
    vec![
        "diff",
        "merge",
        "highlight",
        "backup",
    ]
}

pub fn footer() -> &'static str {
    "right footer"
}

pub fn metrics() -> (u32, u32) {
    (11, 21)
}
