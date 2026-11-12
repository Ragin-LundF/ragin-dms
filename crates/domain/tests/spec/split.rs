use dms_domain::split::{SplitPart, SplitProblem, SplitProposal};

fn part(first: usize, last: usize) -> SplitPart {
    SplitPart { title: Some(format!("Part {first}")), ..SplitPart::pages(first, last) }
}

#[test]
fn parts_that_cover_every_page_in_order_are_kept() {
    let parts = vec![part(0, 1), part(2, 2), part(3, 6)];

    let proposal = SplitProposal::checked(parts.clone(), 7);

    assert_eq!(proposal, SplitProposal { parts, problem: None });
}

#[test]
fn gaps_overlaps_and_pages_beyond_the_end_fall_back_to_one_part_with_the_reason() {
    let cases = [
        (vec![], SplitProblem::NoParts),
        (vec![part(0, 1), part(3, 4)], SplitProblem::Gap),
        (vec![part(1, 4)], SplitProblem::Gap),
        (vec![part(0, 2)], SplitProblem::Gap),
        (vec![part(0, 2), part(2, 4)], SplitProblem::Overlap),
        (vec![part(0, 2), part(4, 3)], SplitProblem::Overlap),
        (vec![part(2, 4), part(0, 1)], SplitProblem::Gap),
        (vec![part(0, 5)], SplitProblem::BeyondLastPage),
    ];
    for (parts, problem) in cases {
        let proposal = SplitProposal::checked(parts.clone(), 5);

        assert_eq!(proposal.problem, Some(problem), "{parts:?}");
        assert_eq!(proposal.parts, vec![SplitPart::pages(0, 4)], "{parts:?}");
    }
}

#[test]
fn a_part_has_a_suggestion_only_when_the_ai_read_something_about_it() {
    assert!(!SplitPart::pages(0, 3).has_suggestion());
    assert!(
        SplitPart { correspondent: Some("Stadtwerke Beispielstadt".to_owned()), ..SplitPart::pages(0, 3) }
            .has_suggestion()
    );
}
