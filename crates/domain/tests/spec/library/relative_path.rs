use dms_domain::library::RelativePath;

use crate::test_data::path;

#[test]
fn decomposed_and_composed_umlauts_give_the_same_path() {
    let decomposed = "A\u{308}rger/U\u{308}bersicht.pdf";
    let composed = "\u{c4}rger/\u{dc}bersicht.pdf";

    assert_eq!(path(decomposed), path(composed));
}

#[test]
fn segments_file_name_and_parent_describe_the_path() {
    let report = path("projects/alpha/report.pdf");

    assert_eq!(report.segments(), vec!["projects", "alpha", "report.pdf"]);
    assert_eq!(report.file_name(), "report.pdf");
    assert_eq!(report.parent(), Some(path("projects/alpha")));
}

#[test]
fn root_has_no_segments_and_no_parent() {
    assert!(RelativePath::root().is_root());
    assert_eq!(RelativePath::root().segments(), [] as [&str; 0]);
    assert_eq!(RelativePath::root().parent(), None);
    assert_eq!(path("top.pdf").parent(), Some(RelativePath::root()));
}

#[test]
fn a_trailing_separator_is_ignored() {
    assert_eq!(path("projects/"), path("projects"));
}

#[test]
fn absolute_paths_and_dot_segments_are_rejected() {
    for raw in ["/etc/passwd", "C:/data", "c:report", "a//b", "a/./b", "a/../b", ".."] {
        assert!(RelativePath::of(raw).is_err(), "{raw}");
    }
}

#[test]
fn segments_from_the_file_system_may_contain_backslashes() {
    let odd = RelativePath::from_segments(&["folder", "odd\\name.txt"]).expect("valid segments");

    assert_eq!(odd.file_name(), "odd\\name.txt");
}

#[test]
fn is_inside_is_strict_and_respects_segment_boundaries() {
    let folder = path("projects");

    assert!(path("projects/a.pdf").is_inside(&folder));
    assert!(!path("projects-old/a.pdf").is_inside(&folder));
    assert!(!folder.is_inside(&folder));
    assert!(folder.is_inside(&RelativePath::root()));
}

#[test]
fn relocate_moves_a_path_from_one_folder_to_another() {
    let moved = path("projects/alpha/report.pdf").relocate(&path("projects"), &path("archive/2024"));

    assert_eq!(moved, Ok(path("archive/2024/alpha/report.pdf")));
}

#[test]
fn relocate_refuses_a_path_outside_the_source_folder() {
    assert!(path("other/a.pdf").relocate(&path("projects"), &path("archive")).is_err());
}

#[test]
fn parse_accepts_what_of_accepts_and_gives_nothing_for_absolute_or_broken_paths() {
    assert_eq!(RelativePath::parse("Letters/2024/"), Some(path("Letters/2024/")));
    for raw in ["/abs", "C:/x", "a//b", "a/../b", "./a"] {
        assert_eq!(RelativePath::parse(raw), None, "{raw}");
    }
}
