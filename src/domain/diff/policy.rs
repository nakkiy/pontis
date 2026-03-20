#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEndingPolicy {
    Compare,
    Ignore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhitespacePolicy {
    Compare,
    Ignore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiffComparePolicies {
    pub whitespace_policy: WhitespacePolicy,
    pub line_ending_policy: LineEndingPolicy,
}

impl DiffComparePolicies {
    pub const fn new(
        whitespace_policy: WhitespacePolicy,
        line_ending_policy: LineEndingPolicy,
    ) -> Self {
        Self {
            whitespace_policy,
            line_ending_policy,
        }
    }

    pub const fn compare() -> Self {
        Self::new(WhitespacePolicy::Compare, LineEndingPolicy::Compare)
    }

    pub const fn with_whitespace(whitespace_policy: WhitespacePolicy) -> Self {
        Self::new(whitespace_policy, LineEndingPolicy::Compare)
    }
}
