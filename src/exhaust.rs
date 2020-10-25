//! Implementation of shuffle multiple slices in its own order
//!
//! This algorithm use backtracking, but loop will be better.

/// Combine all slices in its own order, return all possible cases.
pub fn shuffle<T: Copy + std::fmt::Debug>(inputs: &[&[T]]) -> Vec<Vec<T>> {
    let mut results: Vec<Vec<T>> = Vec::new();
    let mut result: Vec<T> = Vec::new();

    let size = inputs.iter().map(|p| {
        (&p).iter().count()
    }).sum::<usize>();

    let mut used = vec![0; inputs.len()];
    backtracking(&mut results, &mut result, inputs, size, &mut used);
    results
}

fn backtracking<T: Copy>(
    results: &mut Vec<Vec<T>>,
    result: &mut Vec<T>,
    inputs: &[&[T]],
    size: usize,
    used: &mut Vec<usize>,
) {
    if result.len() == size {
        results.push(result.to_vec());
        return;
    }

    for k in 0..used.len() {

        if used[k] >= inputs[k].len() {
            continue;
        }

        result.push(inputs[k][used[k]]);
        used[k] += 1;
        backtracking(results, result, inputs, size, used);
        result.pop();
        used[k] -= 1;
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn exhaust() {
        let cases = vec![
            (vec![], vec![vec![]]),

            (vec![&[1, 2][..]], vec![vec![1, 2]]),

            (vec![&[1, 2][..], &[][..]], vec![vec![1, 2]]),

            (vec![&[1][..], &[8][..]], vec![
                 vec![1, 8],
                 vec![8, 1],
            ]),

            (vec![&[1, 2][..], &[8][..]], vec![
                vec![1, 2, 8],
                vec![1, 8, 2],
                vec![8, 1, 2],
            ]),

            (vec![&[1, 2][..], &[8, 6][..]], vec![
                vec![1, 2, 8, 6],
                vec![1, 8, 6, 2],
                vec![1, 8, 2, 6],
                vec![8, 6, 1, 2],
                vec![8, 1, 6, 2],
                vec![8, 1, 2, 6],
            ]),

            (vec![&[1, 2][..], &[5][..], &[8, 6][..]], vec![
                vec![1, 2, 8, 6, 5],
                vec![1, 2, 8, 5, 6],
                vec![1, 2, 5, 8, 6],
                vec![1, 5, 2, 8, 6],
                vec![5, 1, 2, 8, 6],

                vec![1, 8, 6, 2, 5],
                vec![1, 8, 6, 5, 2],
                vec![1, 8, 5, 6, 2],
                vec![1, 5, 8, 6, 2],
                vec![5, 1, 8, 6, 2],

                vec![1, 8, 2, 6, 5],
                vec![1, 8, 2, 5, 6],
                vec![1, 8, 5, 2, 6],
                vec![1, 5, 8, 2, 6],
                vec![5, 1, 8, 2, 6],

                vec![8, 6, 1, 2, 5],
                vec![8, 6, 1, 5, 2],
                vec![8, 6, 5, 1, 2],
                vec![8, 5, 6, 1, 2],
                vec![5, 8, 6, 1, 2],

                vec![8, 1, 6, 2, 5],
                vec![8, 1, 6, 5, 2],
                vec![8, 1, 5, 6, 2],
                vec![8, 5, 1, 6, 2],
                vec![5, 8, 1, 6, 2],

                vec![8, 1, 2, 6, 5],
                vec![8, 1, 2, 5, 6],
                vec![8, 1, 5, 2, 6],
                vec![8, 5, 1, 2, 6],
                vec![5, 8, 1, 2, 6],
            ]),
        ];

        for (input, mut check) in cases {
            let mut results = super::shuffle(&input);
            let slice = results.as_mut_slice();
            slice.sort();

            let check = check.as_mut_slice();
            check.sort();
            itertools::assert_equal(slice, check);
        }
    }
}
