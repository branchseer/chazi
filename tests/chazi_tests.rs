#[chazi::test(check_reach)]
fn test_top_level_fn() {
    chazi::reached::last();
}

#[chazi::test(ignore)]
fn test_top_level_ignore() {
    panic!()
}

mod tests {
    use std::time::Duration;

    #[chazi::test(exit_code = 12)]
    fn test_exit_code() {
        std::process::exit(12);
    }

    #[chazi::test(exit_code = 12, parent_should_panic)]
    fn test_exit_code_no_matched() {
        std::process::exit(0);
    }

    #[chazi::test(ignore)]
    fn test_ignore() {
        panic!()
    }

    #[chazi::test]
    fn test_basic() {}

    #[chazi::test(parent_should_panic)]
    fn test_no_check_reach() {
        chazi::reached::last();
    }

    #[chazi::test(should_panic)]
    fn test_should_panic() {
        panic!()
    }

    #[chazi::test(check_reach, parent_should_panic)]
    fn test_check_reach_empty() {}

    #[chazi::test(check_reach, parent_should_panic)]
    fn test_reached_last_not_called() {
        chazi::reached::nth(0);
        chazi::reached::nth(1);
    }

    #[chazi::test(check_reach, parent_should_panic)]
    fn test_reached_not_start_with_0() {
        chazi::reached::nth(1);
        chazi::reached::last();
    }

    #[chazi::test(check_reach, parent_should_panic)]
    fn test_reached_wrong_order() {
        chazi::reached::nth(0);
        chazi::reached::nth(2);
    }

    #[chazi::test(check_reach)]
    fn test_reached_last_only() {
        chazi::reached::last();
    }

    #[chazi::test(check_reach)]
    fn test_reached_correct_order() {
        chazi::reached::nth(0);
        chazi::reached::nth(1);
        chazi::reached::last();
    }

    #[chazi::test(check_reach, parent_should_panic)]
    fn test_never() {
        chazi::reached::never();
    }

    #[chazi::test(timeout_ms = 100, parent_should_panic)]
    fn test_timeout() {
        std::thread::sleep(Duration::from_secs(1));
        chazi::reached::last();
    }

    #[chazi::test(check_reach, timeout_ms = 0)]
    fn test_no_timeout() {
        chazi::reached::last();
    }
}
