pub fn gcd(number1: i32, number2: i32) -> i32 {
    if number2 == 0 { number1 } else { gcd(number2, number1 % number2) }
}

pub fn lcm(number1: i32, number2: i32) -> i64 {
    if number1 == 0 || number2 == 0 {
        0
    } else {
        (number1 as i64 * number2 as i64).abs() / gcd(number1, number2) as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(48, 18), 6);
        assert_eq!(gcd(101, 10), 1);
        assert_eq!(gcd(0, 5), 5);
        assert_eq!(gcd(5, 0), 5);
    }

    #[test]
    fn test_lcm() {
        assert_eq!(lcm(4, 6), 12);
        assert_eq!(lcm(21, 6), 42);
        assert_eq!(lcm(0, 5), 0);
        assert_eq!(lcm(5, 0), 0);
    }
}