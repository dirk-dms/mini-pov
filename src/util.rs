macro_rules! block_while {
    ($condition:expr) => {
        while $condition {}
    };
}

macro_rules! block_until {
    ($condition:expr) => {
        block_while!(!$condition)
    };
}
