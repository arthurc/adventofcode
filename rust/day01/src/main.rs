/// Day 1 of Advent of Code 2019
/// https://adventofcode.com/2019/day/1#part2
use std::io::{BufRead, BufReader, Read};

fn main() {
    let f = std::fs::File::open(std::env::args().nth(1).expect("Could not get arg 1"))
        .expect("Could not open input file");
    let fuel: Fuel = read_mass_line(f).map(calculate_fuel_required).sum();

    println!("{}", fuel);
}

type Mass = u32;
type Fuel = u32;

fn read_mass_line<R>(read: R) -> impl Iterator<Item = Mass>
where
    R: Read,
{
    BufReader::new(read)
        .lines()
        .flat_map(|line| line.unwrap().parse::<Mass>())
}

fn calculate_fuel_required(mass: Mass) -> Fuel {
    return mass / 3 - 2;
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_calculate_fuel_required() {
        assert_eq!(2, calculate_fuel_required(12));
        assert_eq!(2, calculate_fuel_required(14));
        assert_eq!(654, calculate_fuel_required(1969));
        assert_eq!(33583, calculate_fuel_required(100756));
    }
}
