use crate::renderer::Renderer;

/// Macro to assert that merged_lines contains a path in either forward or reverse direction
macro_rules! assert_contains_path {
    ($merged_lines:expr, $($point:expr),+ $(,)?) => {
        {
            let expected_forward = vec![$($point),+];
            let expected_reverse: Vec<[f64; 2]> = expected_forward.iter().rev().cloned().collect();
            assert!(
                $merged_lines.contains(&expected_forward) || $merged_lines.contains(&expected_reverse),
                "Expected path not found in either direction. Expected: {:?} or {:?}, Got: {:?}",
                expected_forward,
                expected_reverse,
                $merged_lines
            );
        }
    };
}

#[test]
fn test_merge_lines_1() {
    let lines = vec![vec![[0.0, 0.0], [1.0, 1.0]], vec![[1.0, 1.0], [2.0, 2.0]]];
    let merged_lines = Renderer::merge_lines(lines.iter().collect());
    assert_eq!(merged_lines, vec![vec![[0.0, 0.0], [1.0, 1.0], [2.0, 2.0]]]);
}

#[test]
fn test_merge_lines_2() {
    let lines = vec![
        vec![[2.75, -1.0], [2.90625, -1.0]],
        vec![[2.90625, -1.25], [2.90625, -1.0]],
        vec![[2.75, -1.25], [2.90625, -1.25]],
    ];
    let merged_lines = Renderer::merge_lines(lines.iter().collect());
    assert_eq!(
        merged_lines,
        vec![vec![
            [2.75, -1.0],
            [2.90625, -1.0],
            [2.90625, -1.25],
            [2.75, -1.25]
        ]]
    );
}

#[test]
fn test_merge_lines_3() {
    let lines = vec![
        vec![[2.75, -1.375], [2.75, -1.25]],
        vec![[2.75, -1.0], [2.90625, -1.0]],
        vec![[2.90625, -1.25], [2.90625, -1.0]],
        vec![[2.75, -1.25], [2.90625, -1.25]],
        vec![[2.75, -1.25], [2.75, -1.125]],
    ];
    let merged_lines = Renderer::merge_lines(lines.iter().collect());
    assert_contains_path!(
        merged_lines,
        [2.75, -1.0],
        [2.90625, -1.0],
        [2.90625, -1.25],
        [2.75, -1.25]
    );
    assert_contains_path!(merged_lines, [2.75, -1.375], [2.75, -1.25]);
    assert_contains_path!(merged_lines, [2.75, -1.25], [2.75, -1.125]);
}

#[test]
fn test_merge_lines_3_reordered() {
    let lines = vec![
        vec![[2.75, -1.375], [2.75, -1.25]],
        vec![[2.90625, -1.25], [2.90625, -1.0]],
        vec![[2.75, -1.0], [2.90625, -1.0]],
        vec![[2.75, -1.25], [2.90625, -1.25]],
        vec![[2.75, -1.25], [2.75, -1.125]],
    ];
    let merged_lines = Renderer::merge_lines(lines.iter().collect());
    assert_contains_path!(
        merged_lines,
        [2.75, -1.0],
        [2.90625, -1.0],
        [2.90625, -1.25],
        [2.75, -1.25]
    );
    assert_contains_path!(merged_lines, [2.75, -1.375], [2.75, -1.25]);
    assert_contains_path!(merged_lines, [2.75, -1.25], [2.75, -1.125]);
}

#[test]
fn test_merge_lines_4() {
    let lines = vec![
        vec![[1.90625, -1.40625], [2.125, -1.40625]],
        vec![[4.90625, -1.40625], [5.125, -1.40625]],
        vec![[-1.0625, -1.6875], [-1.0625, -1.65625]],
        vec![[-1.125, -1.65625], [-1.0625, -1.65625]],
        vec![[-1.125, -1.65625], [-1.125, -1.625]],
        vec![[-1.25, -1.625], [-1.125, -1.625]],
    ];
    let merged_lines = Renderer::merge_lines(lines.iter().collect());
    assert_contains_path!(merged_lines, [1.90625, -1.40625], [2.125, -1.40625]);
    assert_contains_path!(merged_lines, [4.90625, -1.40625], [5.125, -1.40625]);
    assert_contains_path!(
        merged_lines,
        [-1.0625, -1.6875],
        [-1.0625, -1.65625],
        [-1.125, -1.65625],
        [-1.125, -1.625],
        [-1.25, -1.625]
    );
}

#[test]
fn test_merge_lines_5() {
    let lines = vec![
        vec![[4.0, -1.375], [4.0, -1.25]],
        vec![[3.8125, -1.25], [3.8125, -1.0]],
        vec![[3.8125, -1.0], [4.0, -1.0]],
        vec![[3.8125, -1.25], [4.0, -1.25]],
        vec![[4.0, -1.25], [4.0, -1.125]],
    ];
    let merged_lines = Renderer::merge_lines(lines.iter().collect());
    assert_contains_path!(merged_lines, [4.0, -1.375], [4.0, -1.25]);
    assert_contains_path!(merged_lines, [4.0, -1.25], [4.0, -1.125]);
    assert_contains_path!(
        merged_lines,
        [4.0, -1.0],
        [3.8125, -1.0],
        [3.8125, -1.25],
        [4.0, -1.25]
    );
}
