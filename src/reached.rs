use std::sync::atomic::{AtomicBool, Ordering};

use crate::probe::generate_probe;

pub(crate) static CHECK_REACH: AtomicBool = AtomicBool::new(false);

fn assert_check_reach() {
    if !CHECK_REACH.load(Ordering::Relaxed) {
        panic!(
            "Functions under chazi::reached should be called only in tests marked with #[chazi::test(check_reach, ...)]"
        )
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub(crate) enum ReachLineInfo {
    Nth(u16),
    Last,
    Never,
}

impl ToString for ReachLineInfo {
    fn to_string(&self) -> String {
        match self {
            ReachLineInfo::Nth(nth_num) => nth_num.to_string(),
            ReachLineInfo::Last => "$".to_string(),
            ReachLineInfo::Never => "!".to_string(),
        }
    }
}

pub(crate) fn parse_reach_info(content: &str) -> Option<ReachLineInfo> {
    match content {
        "$" => Some(ReachLineInfo::Last),
        "!" => Some(ReachLineInfo::Never),
        nth_str => Some(ReachLineInfo::Nth(nth_str.parse().ok()?)),
    }
}

pub(crate) fn write_reach_probe(info: ReachLineInfo) {
    let info_str = info.to_string();
    eprintln!("{}", generate_probe(info_str.as_str()));
}

pub fn nth(nth_num: u16) {
    assert_check_reach();
    write_reach_probe(ReachLineInfo::Nth(nth_num))
}

pub fn last() {
    assert_check_reach();
    write_reach_probe(ReachLineInfo::Last)
}

pub fn never() {
    assert_check_reach();
    write_reach_probe(ReachLineInfo::Never)
}

pub(crate) fn validate_reaches(reach_lines: &[ReachLineInfo]) {
    let mut last_reach_line: Option<ReachLineInfo> = None;
    use ReachLineInfo::{Last, Never, Nth};
    for reach_line in reach_lines {
        match (&last_reach_line, reach_line) {
            (_, Never) => {
                panic!("chazi::reached::never() encountered {:?}", reach_lines)
            }
            (Some(Never), _) => {
                panic!("chazi::reached::never() encountered {:?}", reach_lines)
            }
            (None, Nth(nth)) => {
                assert_eq!(0, *nth, "Incorrect reach order {:?}", reach_lines)
            }
            (Some(Last), _) => {
                panic!("Incorrect reach order {:?}", reach_lines)
            }
            (Some(Nth(last_nth)), Nth(nth)) => {
                assert_eq!(
                    *last_nth + 1,
                    *nth,
                    "Incorrect reach order {:?}",
                    reach_lines
                )
            }
            (Some(Nth(_)), Last) => {}
            (None, Last) => {}
        }
        last_reach_line = Some(reach_line.clone());
    }
    assert_eq!(
        last_reach_line,
        Some(Last),
        "Incorrect reach order {:?}",
        reach_lines
    )
}
