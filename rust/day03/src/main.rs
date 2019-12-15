use std::io::{BufRead, BufReader, Read};

fn main() {
    let f = std::fs::File::open(std::env::args().nth(1).expect("Could not get arg 1"))
        .expect("Could not open input file");
    println!("{:?}", find_shortest_path(f));
}

fn find_shortest_path<R>(read: R) -> Option<(Point, u32)>
where
    R: Read,
{
    let (path2, path1) = {
        let wires = read_wires(read);
        let mut paths: Vec<Path> = wires
            .iter()
            .map(|wire| commands_to_path(&wire, Point::ZERO))
            .collect();
        (paths.pop().unwrap(), paths.pop().unwrap())
    };

    let (lines1, lines2) = (
        points_to_lines(&path1, vec![]),
        points_to_lines(&path2, vec![]),
    );

    let mut intersecting_points = intersecting_points(&lines1, &lines2);
    intersecting_points.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    intersecting_points.reverse();

    intersecting_points.pop()
}

#[derive(Debug, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}
type Distance = u32;
#[derive(PartialEq, Debug)]
struct Command(Direction, Distance);
type Wire = Vec<Command>;
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
struct Point(i32, i32);
impl Point {
    const ZERO: Point = Point(0, 0);

    fn manhattan_distance_to(&self, other: &Point) -> u32 {
        ((other.0 - self.0).abs() + (other.1 - self.1).abs()) as u32
    }
}
type Path = Vec<Point>;
#[derive(Debug, PartialEq, Clone)]
struct Line(Point, Point);
impl Line {
    /// Intersection implemented as per https://en.m.wikipedia.org/wiki/Line–line_intersection#Given_two_points_on_each_line
    fn intersects(&self, other: &Line) -> Option<Point> {
        let Point(x1, y1) = self.0;
        let Point(x2, y2) = self.1;
        let Point(x3, y3) = other.0;
        let Point(x4, y4) = other.1;

        let tnn = (x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4);
        let tdn = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
        let unn = (x1 - x2) * (y1 - y3) - (y1 - y2) * (x1 - x3);

        if tdn == 0 || tnn == 0 || unn == 0 {
            return None;
        } else {
            let t = tnn as f64 / tdn as f64;
            let u = -unn as f64 / tdn as f64;

            // println!("t: {}, u: {}", t, u);

            if !((0f64 < t && t < 1f64) && (0f64 < u && u < 1f64)) {
                return None;
            }

            let px = x1 + (t * (x2 - x1) as f64) as i32;
            let py = y1 + (t * (y2 - y1) as f64) as i32;

            //println!("L1: {:?}, L2: {:?}", self, other);

            Some(Point(px, py))
        }
    }
}

fn read_wires<R>(read: R) -> Vec<Wire>
where
    R: Read,
{
    BufReader::new(read)
        .lines()
        .map(|l| read_commands(&l.unwrap()))
        .filter(|l| !l.is_empty())
        .collect()
}

fn read_commands(line: &str) -> Vec<Command> {
    line.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(parse_command)
        .collect()
}

fn parse_command(s: &str) -> Command {
    let direction = parse_direction(s.chars().next().unwrap());
    let distance = s[1..].parse::<Distance>().unwrap();

    Command(direction, distance)
}

fn parse_direction(c: char) -> Direction {
    match c {
        'L' => Direction::Left,
        'R' => Direction::Right,
        'U' => Direction::Up,
        'D' => Direction::Down,
        c @ _ => panic!("Unparsable: {}", c),
    }
}

fn commands_to_path(commands: &[Command], p: Point) -> Path {
    let path = vec![p];

    if commands.is_empty() {
        return path;
    }

    let Point(px, py) = p;
    let p2 = match commands[0] {
        Command(Direction::Left, x) => Point(px - x as i32, py),
        Command(Direction::Right, x) => Point(px + x as i32, py),
        Command(Direction::Up, y) => Point(px, py + y as i32),
        Command(Direction::Down, y) => Point(px, py - y as i32),
    };

    [&path[..], &commands_to_path(&commands[1..], p2)[..]].concat()
}

fn points_to_lines(points: &[Point], mut lines: Vec<Line>) -> Vec<Line> {
    match (points.get(0), points.get(1)) {
        (Some(p1), Some(p2)) => {
            lines.push(Line(p1.clone(), p2.clone()));
            points_to_lines(&points[1..], lines)
        }
        _ => lines,
    }
}

fn intersecting_points(lines1: &Vec<Line>, lines2: &Vec<Line>) -> Vec<(Point, u32)> {
    let mut v = vec![];
    for line1 in lines1 {
        for line2 in lines2 {
            if let Some(p) = line1.intersects(&line2) {
                v.push((p, p.manhattan_distance_to(&Point::ZERO)));
            }
        }
    }
    v
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::io::Cursor;
    use Direction::*;

    #[test]
    fn test_parse_direction() {
        assert_eq!(Right, parse_direction('R'));
        assert_eq!(Left, parse_direction('L'));
        assert_eq!(Up, parse_direction('U'));
        assert_eq!(Down, parse_direction('D'));
    }

    #[test]
    fn test_parse_command() {
        assert_eq!(Command(Right, 10), parse_command("R10"));
        assert_eq!(Command(Up, 9), parse_command("U9"));
        assert_eq!(Command(Down, 999), parse_command("D999"));
        assert_eq!(Command(Left, 7), parse_command("L7"));
    }

    #[test]
    fn test_commands_to_path() {
        let path = commands_to_path(
            &[Command(Up, 10), Command(Right, 9), Command(Down, 1)],
            Point::ZERO,
        );

        assert_eq!(
            vec![Point(0, 0), Point(0, 10), Point(9, 10), Point(9, 9)],
            path
        );
    }

    #[test]
    fn test_points_to_lines() {
        assert_eq!(
            vec![
                Line(Point(1, 1), Point(2, 2)),
                Line(Point(2, 2), Point(3, 4))
            ],
            points_to_lines(&[Point(1, 1), Point(2, 2), Point(3, 4)], vec![])
        );
    }

    #[test]
    fn test_line_intersects() {
        assert_eq!(
            Some(Point(0, 0)),
            Line(Point(0, -1), Point(0, 1)).intersects(&Line(Point(-1, 0), Point(1, 0)))
        );
        assert_eq!(
            None,
            Line(Point(1, 0), Point(1, 2)).intersects(&Line(Point(2, 0), Point(2, 2)))
        );
        assert_eq!(
            None,
            Line(Point(0, 0), Point(0, 7)).intersects(&Line(Point(0, 0), Point(8, 0)))
        );
        assert_eq!(
            None,
            Line(Point(0, 0), Point(8, 0)).intersects(&Line(Point(6, 7), Point(6, 3)))
        );
        assert_eq!(
            None,
            Line(Point(8, 0), Point(8, 5)).intersects(&Line(Point(6, 3), Point(2, 3)))
        );
    }

    #[test]
    fn test_read_wires_returns_an_empty_vec_if_empty_line() {
        assert_eq!(Vec::<Wire>::new(), read_wires(Cursor::new(" ")));
    }

    #[test]
    fn test_example1() {
        let s = "
        R8,U5,L5,D3
        U7,R6,D4,L4
        ";

        let mut wires = read_wires(Cursor::new(s));
        let wire2 = wires.pop().unwrap();
        let wire1 = wires.pop().unwrap();
        assert_eq!(
            vec![
                Command(Right, 8),
                Command(Up, 5),
                Command(Left, 5),
                Command(Down, 3)
            ],
            wire1
        );
        assert_eq!(
            vec![
                Command(Up, 7),
                Command(Right, 6),
                Command(Down, 4),
                Command(Left, 4)
            ],
            wire2
        );

        let path1 = commands_to_path(&wire1, Point::ZERO);
        let path2 = commands_to_path(&wire2, Point::ZERO);
        assert_eq!(
            vec![
                Point(0, 0),
                Point(8, 0),
                Point(8, 5),
                Point(3, 5),
                Point(3, 2)
            ],
            path1
        );
        assert_eq!(
            vec![
                Point(0, 0),
                Point(0, 7),
                Point(6, 7),
                Point(6, 3),
                Point(2, 3)
            ],
            path2
        );

        let lines1 = points_to_lines(&path1, vec![]);
        let lines2 = points_to_lines(&path2, vec![]);
        assert_eq!(
            vec![
                Line(Point(0, 0), Point(8, 0)),
                Line(Point(8, 0), Point(8, 5)),
                Line(Point(8, 5), Point(3, 5)),
                Line(Point(3, 5), Point(3, 2))
            ],
            lines1
        );
        assert_eq!(
            vec![
                Line(Point(0, 0), Point(0, 7)),
                Line(Point(0, 7), Point(6, 7)),
                Line(Point(6, 7), Point(6, 3)),
                Line(Point(6, 3), Point(2, 3))
            ],
            lines2
        );

        let intersecting_points = intersecting_points(&lines1, &lines2);
        assert_eq!(
            vec![(Point(6, 5), 11), (Point(3, 3), 6)],
            intersecting_points
        );
        //let (_, x) = find_shortest_path().unwrap();

        //assert_eq!(6, x);
    }

    #[test]
    fn test_example2() {
        let s = "
        R75,D30,R83,U83,L12,D49,R71,U7,L72
        U62,R66,U55,R34,D71,R55,D58,R83
        ";

        let (_, x) = find_shortest_path(Cursor::new(s)).unwrap();

        assert_eq!(159, x);
    }

    #[test]
    fn test_example3() {
        let s = "
        R98,U47,R26,D63,R33,U87,L62,D20,R33,U53,R51
        U98,R91,D20,R16,D67,R40,U7,R15,U6,R7
        ";

        let (_, x) = find_shortest_path(Cursor::new(s)).unwrap();

        assert_eq!(135, x);
    }
}
