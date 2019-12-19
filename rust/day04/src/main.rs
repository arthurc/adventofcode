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

#[derive(PartialEq, Debug)]
enum NumbersError {
    LengthNot6(usize),
    NumbersNotAllIncreasing,
    NoTwoNumbersSuccessive,
}

type NumbersResult<T> = Result<T, NumbersError>;

#[derive(Debug, PartialEq)]
struct Numbers([u32; 6]);
impl Numbers {
    fn new(n: &[u32]) -> NumbersResult<Numbers> {
        if n.len() != 6 {
            return Err(NumbersError::LengthNot6(n.len()));
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
            return Err(NumbersError::NumbersNotAllIncreasing);
        }

        // Look for invalid digit groups
        if !count_digits(n).iter().any(|&(_, count)| count == 2) {
            return Err(NumbersError::NoTwoNumbersSuccessive);
        }

        Ok(Numbers(slice))
    }

    fn parse(s: &str) -> NumbersResult<Numbers> {
        // Construct a slice out of the string
        Numbers::new(
            &s.chars()
                .map(|c| c.to_digit(10).unwrap())
                .collect::<Vec<_>>()[..],
        )
    }

    fn parse_pair(s1: &str, s2: &str) -> NumbersResult<(Numbers, Numbers)> {
        Ok((Numbers::parse(s1)?, Numbers::parse(s2)?))
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

fn count_digits(slice: &[u32]) -> Vec<(u32, usize)> {
    slice.iter().fold(vec![], |mut acc, &n| {
        match acc.last_mut() {
            Some((acc_digit, acc_count)) if *acc_digit == n => *acc_count += 1,
            _ => acc.push((n, 1)),
        };
        acc
    })
}

#[cfg(test)]
mod numbers_tests {

    use super::*;

    #[test]
    fn parse_should_return_none_if_the_length_is_not_6() {
        assert_eq!(Err(NumbersError::LengthNot6(7)), Numbers::parse("1234567"));
        assert_eq!(Err(NumbersError::LengthNot6(5)), Numbers::parse("12345"));
    }

    #[test]
    fn parse_should_be_able_to_parse_a_string() {
        assert_eq!(Ok(Numbers([1, 1, 3, 4, 5, 9])), Numbers::parse("113459"));
    }

    #[test]
    fn parse_pair_should_be_able_to_parse_a_pair_of_strings() {
        assert_eq!(
            Ok((Numbers([1, 1, 3, 4, 5, 6]), Numbers([1, 1, 3, 4, 5, 6]))),
            Numbers::parse_pair("113456", "113456")
        );
    }

    #[test]
    fn new_should_return_none_if_the_numbers_are_not_increasing() {
        assert_eq!(
            Err(NumbersError::NumbersNotAllIncreasing),
            Numbers::new(&[1, 2, 3, 4, 7, 6])
        );
    }

    #[test]
    fn new_should_return_a_value_if_its_valid() {
        assert_eq!(
            Ok(Numbers([1, 1, 3, 4, 5, 6])),
            Numbers::new(&[1, 1, 3, 4, 5, 6])
        );
    }

    #[test]
    fn new_should_return_none_if_no_duplicates_are_found() {
        assert_eq!(
            Err(NumbersError::NoTwoNumbersSuccessive),
            Numbers::new(&[1, 2, 3, 4, 5, 6])
        );
    }

    #[test]
    fn test_parse_examples() {
        assert_eq!(
            Err(NumbersError::NoTwoNumbersSuccessive),
            Numbers::parse("111111")
        );
        assert_eq!(
            Err(NumbersError::NumbersNotAllIncreasing),
            Numbers::parse("223450")
        );
        assert_eq!(
            Err(NumbersError::NoTwoNumbersSuccessive),
            Numbers::parse("123789")
        );
    }

    #[test]
    fn to_u32_should_return_the_number_as_u32() {
        assert_eq!(112234, Numbers::parse("112234").unwrap().to_u32());
    }

    #[test]
    fn test_parse_examples_part2() {
        assert!(Numbers::parse("112233").is_ok());
        assert_eq!(
            Err(NumbersError::NoTwoNumbersSuccessive),
            Numbers::parse("123444")
        );
        assert!(Numbers::parse("111122").is_ok());
    }
}

#[cfg(test)]
mod group_tests {

    use super::*;

    #[test]
    fn test_count_digits() {
        assert_eq!(
            vec![(1, 2), (2, 2), (3, 4), (7, 1)],
            count_digits(&[1, 1, 2, 2, 3, 3, 3, 3, 7])
        );
    }
}
