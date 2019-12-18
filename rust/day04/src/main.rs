use std::fmt;

fn main() {
    let (n1, n2) = (
        std::env::args().nth(1).unwrap().parse::<u32>().unwrap(),
        std::env::args().nth(2).unwrap().parse::<u32>().unwrap(),
    );

    let count = (n1..n2)
        .into_iter()
        .map(|i| Numbers::parse(&i.to_string()))
        .flatten()
        .count();

    println!("{}", count);
}

#[derive(Debug, PartialEq)]
struct Numbers([u32; 6]);
impl Numbers {
    fn new(n: &[u32]) -> Option<Numbers> {
        if n.len() != 6 {
            return None;
        }

        let mut slice = [0u32; 6];
        slice.copy_from_slice(n);

        // Check that all numbers are increasing
        let mut iter = n.iter();
        if !iter
            .next()
            .map(|&n| {
                iter.fold((true, n), |(accb, accn), &n| (accb && n >= accn, n))
                    .0
            })
            .unwrap_or(true)
        {
            return None;
        }

        // Check that there is atleast two numbers are successive
        let mut iter = n.iter();
        if !iter
            .next()
            .map(|&n| {
                iter.fold((false, n), |(accb, accn), &n| (accb || accn == n, n))
                    .0
            })
            .unwrap_or(true)
        {
            return None;
        }

        Some(Numbers(slice))
    }

    fn parse(s: &str) -> Option<Numbers> {
        // Construct a slice out of the string
        Numbers::new(
            &s.chars()
                .map(|c| c.to_digit(10).unwrap())
                .collect::<Vec<_>>()[..],
        )
    }

    fn parse_pair(s1: &str, s2: &str) -> Option<(Numbers, Numbers)> {
        Some((Numbers::parse(s1)?, Numbers::parse(s2)?))
    }

    fn to_u32(&self) -> u32 {
        self.0
            .iter()
            .rev()
            .enumerate()
            .fold(0, |acc, (idx, n)| acc + n * 10u32.pow(idx as u32))
    }
}
impl fmt::Display for Numbers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_u32())
    }
}

#[cfg(test)]
mod numbers_tests {

    use super::*;

    #[test]
    fn parse_should_return_none_if_the_length_is_not_6() {
        assert_eq!(None, Numbers::parse("1234567"));
        assert_eq!(None, Numbers::parse("12345"));
    }

    #[test]
    fn parse_should_be_able_to_parse_a_string() {
        assert_eq!(Some(Numbers([1, 1, 3, 4, 5, 9])), Numbers::parse("113459"));
    }

    #[test]
    fn parse_pair_should_be_able_to_parse_a_pair_of_strings() {
        assert_eq!(
            Some((Numbers([1, 1, 3, 4, 5, 6]), Numbers([1, 1, 3, 4, 5, 6]))),
            Numbers::parse_pair("113456", "113456")
        );
    }

    #[test]
    fn new_should_return_none_if_the_numbers_are_not_increasing() {
        assert_eq!(None, Numbers::new(&[1, 2, 3, 4, 7, 6]));
    }

    #[test]
    fn new_should_return_a_value_if_its_valid() {
        assert_eq!(
            Some(Numbers([1, 1, 3, 4, 5, 6])),
            Numbers::new(&[1, 1, 3, 4, 5, 6])
        );
    }

    #[test]
    fn new_should_return_none_if_no_duplicates_are_found() {
        assert_eq!(None, Numbers::new(&[1, 2, 3, 4, 5, 6]));
    }

    #[test]
    fn test_parse_examples() {
        assert!(Numbers::parse("111111").is_some());
        assert!(Numbers::parse("223450").is_none());
        assert!(Numbers::parse("123789").is_none());
    }

    #[test]
    fn to_u32_should_return_the_number_as_u32() {
        assert_eq!(112234, Numbers::parse("112234").unwrap().to_u32());
    }
}
