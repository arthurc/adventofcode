/// Day 1 of Advent of Code 2019
/// https://adventofcode.com/2019/day/1#part2
use std::io::{BufRead, BufReader, Read};

fn main() {
    let f = std::fs::File::open(std::env::args().nth(1).expect("Could not get arg 1"))
        .expect("Could not open input file");
    let fuel: Fuel = read_mass_line(f)
        .map(calculate_fuel_required_for_mass)
        .sum();

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

fn calculate_fuel_required_for_mass(mass: Mass) -> Fuel {
    let fuel = mass / 3 - 2;
    return fuel + calculate_fuel_required_for_fuel(fuel);
}

fn calculate_fuel_required_for_fuel(fuel: Fuel) -> Fuel {
    match (fuel / 3).saturating_sub(2) {
        n if n <= 0 => 0,
        n => n + calculate_fuel_required_for_fuel(n),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_calculate_fuel_required_for_mass() {
        assert_eq!(2, calculate_fuel_required_for_mass(12));
        assert_eq!(2, calculate_fuel_required_for_mass(14));
        assert_eq!(966, calculate_fuel_required_for_mass(1969));
        assert_eq!(50346, calculate_fuel_required_for_mass(100756));
    }

    #[test]
    fn test_calculate_fuel_required_for_fuel() {
        assert_eq!(0, calculate_fuel_required_for_fuel(2));
        assert_eq!(0, calculate_fuel_required_for_fuel(5));
        assert_eq!(5, calculate_fuel_required_for_fuel(21));
        assert_eq!(26, calculate_fuel_required_for_fuel(70));
        assert_eq!(96, calculate_fuel_required_for_fuel(216));
    }
}
