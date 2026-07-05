pub fn calculate_difference(
    current_state: &[([f32; 9], f32)],
    past_state: &[([f32; 9], f32)],
) -> f32 {
    current_state.iter().zip(past_state.iter()).map(|((current_matrix, current_opacity), (past_matrix, past_opacity))| {
        let mut diff = 0.0;

        diff += (current_matrix[6] - past_matrix[6]).abs();
        diff += (current_matrix[7] - past_matrix[7]).abs();
        diff += (current_matrix[0] - past_matrix[0]).abs() * 100.0;
        diff += (current_matrix[1] - past_matrix[1]).abs() * 100.0;
        diff += (current_matrix[3] - past_matrix[3]).abs() * 100.0;
        diff += (current_matrix[4] - past_matrix[4]).abs() * 100.0;
        diff += (current_opacity - past_opacity).abs() * 255.0;

        diff
    }).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_difference_identical() {
        let state_a = vec![([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 10.0, 20.0, 1.0], 1.0)];
        let state_b = vec![([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 10.0, 20.0, 1.0], 1.0)];

        let diff = calculate_difference(&state_a, &state_b);
        assert_eq!(diff, 0.0);
    }

    #[test]
    fn test_calculate_difference_variance() {
        let state_a = vec![([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 10.0, 20.0, 1.0], 1.0)];
        let state_b = vec![([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 15.0, 20.0, 1.0], 0.5)];

        let diff = calculate_difference(&state_a, &state_b);
        assert_eq!(diff, 132.5);
    }
}