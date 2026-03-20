pub fn title() -> &'static str {
    "pontis"
}

pub fn feature_flags() -> Vec<&'static str> {
    vec![
        "diff",
        "merge",
        "highlight",
    ]
}

pub fn footer() -> &'static str {
    "left footer"
}

pub fn metrics() -> (u32, u32) {
    (10, 20)
}
