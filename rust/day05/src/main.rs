use std::io::prelude::*;
use std::io::{BufRead, BufReader};

fn main() {
    let f = std::fs::File::open(std::env::args().nth(1).expect("Could not get arg 1"))
        .expect("Could not open input file");

    let code: Vec<i32> = read_intcode(f).collect();
    let mut program = Program::new(code);

    while !program.finished {
        program.tick();
    }
}

fn read_intcode<R>(read: R) -> impl Iterator<Item = i32>
where
    R: Read,
{
    BufReader::new(read)
        .split(b',')
        .map(|l| String::from_utf8(l.unwrap()).unwrap())
        .flat_map(|s| s.trim().parse::<i32>())
}

type Pc = usize;
type Code = Vec<i32>;

#[derive(Debug, PartialEq)]
enum Opcode {
    Add(ParameterMode, ParameterMode),
    Mul(ParameterMode, ParameterMode),
    In,
    Out(ParameterMode),
    JT(ParameterMode, ParameterMode),
    JF(ParameterMode, ParameterMode),
    LT(ParameterMode, ParameterMode),
    EQ(ParameterMode, ParameterMode),
    Fin,
}
impl Opcode {
    fn decode(i: i32) -> Option<Opcode> {
        macro_rules! param {
            ($idx:expr) => {{
                let ii = i / (100 * 10i32.pow($idx));
                ParameterMode::parse(ii - 10 * (ii / 10)).unwrap()
            }};
        }

        // Pick out the last two digits to determine the opcode
        match i - 100 * (i / 100) {
            1 => Some(Opcode::Add(param!(0), param!(1))),
            2 => Some(Opcode::Mul(param!(0), param!(1))),
            3 => Some(Opcode::In),
            4 => Some(Opcode::Out(param!(0))),
            5 => Some(Opcode::JT(param!(0), param!(1))),
            6 => Some(Opcode::JF(param!(0), param!(1))),
            7 => Some(Opcode::LT(param!(0), param!(1))),
            8 => Some(Opcode::EQ(param!(0), param!(1))),
            99 => Some(Opcode::Fin),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
struct Instr(Opcode, Vec<i32>);
impl Instr {
    fn decode(code: &[i32]) -> Option<Instr> {
        let opcode = Opcode::decode(code[0])?;

        let params = match opcode {
            Opcode::Add(..) => Some(vec![code[1], code[2], code[3]]),
            Opcode::Mul(..) => Some(vec![code[1], code[2], code[3]]),
            Opcode::In => Some(vec![code[1]]),
            Opcode::Out(..) => Some(vec![code[1]]),
            Opcode::JT(..) => Some(vec![code[1], code[2]]),
            Opcode::JF(..) => Some(vec![code[1], code[2]]),
            Opcode::LT(..) => Some(vec![code[1], code[2], code[3]]),
            Opcode::EQ(..) => Some(vec![code[1], code[2], code[3]]),
            Opcode::Fin => Some(vec![]),
        };

        params.map(|p| Instr(opcode, p))
    }
}

#[derive(Debug, PartialEq)]
enum ParameterMode {
    Position,
    Immediate,
}
impl ParameterMode {
    fn parse(n: i32) -> Option<ParameterMode> {
        match n {
            0 => Some(ParameterMode::Position),
            1 => Some(ParameterMode::Immediate),
            _ => None,
        }
    }
}

#[derive(PartialEq, Debug)]
struct Program {
    pc: Pc,
    code: Code,
    finished: bool,
}
impl Program {
    fn new(code: Code) -> Self {
        Self {
            code,
            pc: 0,
            finished: false,
        }
    }

    fn tick(&mut self) {
        let instr = Instr::decode(&self.code[self.pc..]).expect("Could not decode instruction");
        self.execute_instr(instr, &mut std::io::stdin().lock(), &mut std::io::stdout());
    }

    fn execute_instr(&mut self, instr: Instr, r#in: &mut dyn BufRead, out: &mut dyn Write) {
        use Opcode::*;

        macro_rules! param_v {
            ($params:expr, $pi:expr, $i:expr) => {
                match $pi {
                    ParameterMode::Immediate => $params[$i],
                    ParameterMode::Position => self.code[$params[$i] as usize],
                }
            };
        }

        match instr {
            Instr(Add(pa, pb), params) => {
                let (a, b, t) = (param_v!(params, pa, 0), param_v!(params, pb, 1), params[2]);

                self.code[t as usize] = a + b;
                self.pc += 4;
            }
            Instr(Mul(pa, pb), params) => {
                let (a, b, t) = (param_v!(params, pa, 0), param_v!(params, pb, 1), params[2]);

                self.code[t as usize] = a * b;
                self.pc += 4;
            }
            Instr(In, params) => {
                let t = params[0];

                out.write_fmt(format_args!("Input: ")).unwrap();
                out.flush().unwrap();

                let mut line = String::new();
                r#in.read_line(&mut line).unwrap();

                self.code[t as usize] = line.trim().parse().unwrap();
                self.pc += 2;
            }
            Instr(Out(pa), params) => {
                let a = param_v!(params, pa, 0);

                out.write_fmt(format_args!("{}", a)).unwrap();
                self.pc += 2;
            }
            Instr(JT(pa, pb), params) => {
                let (a, b) = (param_v!(params, pa, 0), param_v!(params, pb, 1));

                self.pc = if a != 0 { b as usize } else { self.pc + 3usize };
            }
            Instr(JF(pa, pb), params) => {
                let (a, b) = (param_v!(params, pa, 0), param_v!(params, pb, 1));

                self.pc = if a == 0 { b as usize } else { self.pc + 3usize };
            }
            Instr(LT(pa, pb), params) => {
                let (a, b, t) = (param_v!(params, pa, 0), param_v!(params, pb, 1), params[2]);

                self.code[t as usize] = if a < b { 1 } else { 0 };
                self.pc += 4;
            }
            Instr(EQ(pa, pb), params) => {
                let (a, b, t) = (param_v!(params, pa, 0), param_v!(params, pb, 1), params[2]);

                self.code[t as usize] = if a == b { 1 } else { 0 };
                self.pc += 4;
            }
            Instr(Fin, ..) => self.finished = true,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_execute_instr_add() {
        let mut program = Program::new(vec![0, 1, 2, 0]);

        program.execute_instr(
            Instr(
                Opcode::Add(ParameterMode::Position, ParameterMode::Position),
                vec![1, 2, 0],
            ),
            &mut Cursor::new(""),
            &mut Cursor::new(Vec::new()),
        );

        assert_eq!(
            Program {
                code: vec![3, 1, 2, 0],
                pc: 4,
                finished: false
            },
            program
        );
    }

    #[test]
    fn test_execute_instr_mul() {
        let mut program = Program::new(vec![0, 4, 5, 0, 5, 6]);

        program.execute_instr(
            Instr(
                Opcode::Mul(ParameterMode::Position, ParameterMode::Position),
                vec![4, 5, 0],
            ),
            &mut Cursor::new(""),
            &mut Cursor::new(Vec::new()),
        );

        assert_eq!(
            Program {
                pc: 4,
                code: vec![30, 4, 5, 0, 5, 6],
                finished: false
            },
            program
        );
    }

    #[test]
    fn test_execute_instr_mul_neg() {
        let mut program = Program::new(vec![0, -4, 5]);

        program.execute_instr(
            Instr(
                Opcode::Mul(ParameterMode::Position, ParameterMode::Position),
                vec![1, 2, 0],
            ),
            &mut Cursor::new(""),
            &mut Cursor::new(Vec::new()),
        );

        assert_eq!(
            Program {
                pc: 4,
                code: vec![-20, -4, 5],
                finished: false
            },
            program
        );
    }

    #[test]
    fn test_execute_instr_fin() {
        let mut program = Program::new(vec![99]);

        program.execute_instr(
            Instr(Opcode::Fin, vec![]),
            &mut Cursor::new(""),
            &mut Cursor::new(Vec::new()),
        );

        assert_eq!(
            Program {
                code: vec![99],
                pc: 0,
                finished: true
            },
            program
        );
    }

    #[test]
    fn test_execute_instr_in() {
        let mut program = Program::new(vec![3, 1]);

        program.execute_instr(
            Instr(Opcode::In, vec![1]),
            &mut Cursor::new("89"),
            &mut Cursor::new(Vec::new()),
        );

        assert_eq!(
            Program {
                code: vec![3, 89],
                pc: 2,
                finished: false
            },
            program
        );
    }

    #[test]
    fn test_execute_instr_out() {
        let mut program = Program::new(vec![4, 10]);
        let mut buf = Cursor::new(Vec::new());

        program.execute_instr(
            Instr(Opcode::Out(ParameterMode::Position), vec![1]),
            &mut Cursor::new(""),
            &mut buf,
        );

        assert_eq!(
            Program {
                code: vec![4, 10],
                pc: 2,
                finished: false
            },
            program
        );
        assert_eq!("10", std::str::from_utf8(buf.get_ref()).unwrap());
    }

    #[test]
    fn test_execute_instr_jt_jump_taken() {
        let mut program = Program::new(vec![5, 1, 7]);

        program.execute_instr(
            Instr(
                Opcode::JT(ParameterMode::Immediate, ParameterMode::Immediate),
                vec![1, 7],
            ),
            &mut Cursor::new(""),
            &mut Cursor::new(Vec::new()),
        );

        assert_eq!(
            Program {
                pc: 7,
                code: vec![5, 1, 7],
                finished: false
            },
            program
        );
    }

    #[test]
    fn test_execute_instr_jt_jump_not_taken() {
        let mut program = Program::new(vec![5, 0, 7]);

        program.execute_instr(
            Instr(
                Opcode::JT(ParameterMode::Immediate, ParameterMode::Immediate),
                vec![0, 7],
            ),
            &mut Cursor::new(""),
            &mut Cursor::new(Vec::new()),
        );

        assert_eq!(
            Program {
                pc: 3,
                code: vec![5, 0, 7],
                finished: false
            },
            program
        );
    }

    fn test_execute_instr_jf_jump_taken() {
        let mut program = Program::new(vec![5, 1, 7]);

        program.execute_instr(
            Instr(
                Opcode::JF(ParameterMode::Immediate, ParameterMode::Immediate),
                vec![1, 7],
            ),
            &mut Cursor::new(""),
            &mut Cursor::new(Vec::new()),
        );

        assert_eq!(
            Program {
                pc: 7,
                code: vec![6, 0, 7],
                finished: false
            },
            program
        );
    }

    #[test]
    fn test_execute_instr_jf_jump_not_taken() {
        let mut program = Program::new(vec![6, 1, 7]);

        program.execute_instr(
            Instr(
                Opcode::JF(ParameterMode::Immediate, ParameterMode::Immediate),
                vec![1, 7],
            ),
            &mut Cursor::new(""),
            &mut Cursor::new(Vec::new()),
        );

        assert_eq!(
            Program {
                pc: 3,
                code: vec![6, 1, 7],
                finished: false
            },
            program
        );
    }

    #[test]
    fn test_execute_instr_lt_true() {
        let mut program = Program::new(vec![8, 1, 7, 0]);

        program.execute_instr(
            Instr(
                Opcode::LT(ParameterMode::Immediate, ParameterMode::Immediate),
                vec![1, 7, 0],
            ),
            &mut Cursor::new(""),
            &mut Cursor::new(Vec::new()),
        );

        assert_eq!(
            Program {
                pc: 4,
                code: vec![1, 1, 7, 0],
                finished: false
            },
            program
        );
    }

    #[test]
    fn test_execute_instr_lt_false() {
        let mut program = Program::new(vec![8, 8, 7, 0]);

        program.execute_instr(
            Instr(
                Opcode::LT(ParameterMode::Immediate, ParameterMode::Immediate),
                vec![8, 7, 0],
            ),
            &mut Cursor::new(""),
            &mut Cursor::new(Vec::new()),
        );

        assert_eq!(
            Program {
                pc: 4,
                code: vec![0, 8, 7, 0],
                finished: false
            },
            program
        );
    }

    #[test]
    fn test_execute_instr_eq_true() {
        let mut program = Program::new(vec![9, 1, 1, 0]);

        program.execute_instr(
            Instr(
                Opcode::EQ(ParameterMode::Immediate, ParameterMode::Immediate),
                vec![1, 1, 0],
            ),
            &mut Cursor::new(""),
            &mut Cursor::new(Vec::new()),
        );

        assert_eq!(
            Program {
                pc: 4,
                code: vec![1, 1, 1, 0],
                finished: false
            },
            program
        );
    }

    #[test]
    fn test_execute_instr_eq_false() {
        let mut program = Program::new(vec![9, 8, 7, 0]);

        program.execute_instr(
            Instr(
                Opcode::EQ(ParameterMode::Immediate, ParameterMode::Immediate),
                vec![8, 7, 0],
            ),
            &mut Cursor::new(""),
            &mut Cursor::new(Vec::new()),
        );

        assert_eq!(
            Program {
                pc: 4,
                code: vec![0, 8, 7, 0],
                finished: false
            },
            program
        );
    }

    #[test]
    fn test_code() {
        let expected = vec![3500, 9, 10, 70, 2, 3, 11, 0, 99, 30, 40, 50];
        let mut program = Program::new(vec![1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50]);

        program.tick();
        program.tick();
        program.tick();

        assert_eq!(
            Program {
                code: expected,
                pc: 8,
                finished: true
            },
            program
        );
    }

    #[test]
    fn execute_instr_with_immediate_mode() {
        let mut program = Program::new(vec![11100, 10, 20, 0]);

        program.execute_instr(
            Instr(
                Opcode::Add(ParameterMode::Immediate, ParameterMode::Immediate),
                vec![10, 20, 0],
            ),
            &mut Cursor::new(""),
            &mut Cursor::new(Vec::new()),
        );

        assert_eq!(
            Program {
                code: vec![30, 10, 20, 0],
                pc: 4,
                finished: false
            },
            program
        );
    }
}

#[cfg(test)]
mod opcode_tests {

    use super::*;

    #[test]
    fn test_decode_opcode() {
        assert_eq!(
            Some(Opcode::Add(
                ParameterMode::Position,
                ParameterMode::Position,
            )),
            Opcode::decode(1)
        );
        assert_eq!(
            Some(Opcode::Mul(
                ParameterMode::Position,
                ParameterMode::Position,
            )),
            Opcode::decode(2)
        );
        assert_eq!(Some(Opcode::In), Opcode::decode(3));
        assert_eq!(
            Some(Opcode::Out(ParameterMode::Position)),
            Opcode::decode(4)
        );
        assert_eq!(Some(Opcode::Fin), Opcode::decode(99));
        assert_eq!(None, Opcode::decode(9));
        assert_eq!(
            Some(Opcode::JT(ParameterMode::Position, ParameterMode::Position)),
            Opcode::decode(5)
        );
        assert_eq!(
            Some(Opcode::JF(ParameterMode::Position, ParameterMode::Position)),
            Opcode::decode(6)
        );
        assert_eq!(
            Some(Opcode::LT(ParameterMode::Position, ParameterMode::Position)),
            Opcode::decode(7)
        );
    }

    #[test]
    fn test_decode_opcode_with_parameters() {
        assert_eq!(
            Some(Opcode::Add(
                ParameterMode::Position,
                ParameterMode::Immediate,
            )),
            Opcode::decode(11001)
        );
    }
}

#[cfg(test)]
mod instr_tests {

    use super::*;

    #[test]
    fn test_decode() {
        assert_eq!(
            Some(Instr(
                Opcode::Add(ParameterMode::Position, ParameterMode::Position,),
                vec![2, 3, 4]
            )),
            Instr::decode(&[1, 2, 3, 4])
        );
        assert_eq!(
            Some(Instr(
                Opcode::Mul(ParameterMode::Position, ParameterMode::Position),
                vec![3, 4, 5]
            )),
            Instr::decode(&[2, 3, 4, 5])
        );
        assert_eq!(Some(Instr(Opcode::Fin, vec![])), Instr::decode(&[99]));
        assert_eq!(Some(Instr(Opcode::In, vec![1])), Instr::decode(&[3, 1]));
        assert_eq!(
            Some(Instr(Opcode::Out(ParameterMode::Position), vec![2])),
            Instr::decode(&[4, 2])
        );
        assert_eq!(
            Some(Instr(
                Opcode::JT(ParameterMode::Position, ParameterMode::Position),
                vec![2, 3]
            )),
            Instr::decode(&[5, 2, 3])
        );
        assert_eq!(
            Some(Instr(
                Opcode::JF(ParameterMode::Position, ParameterMode::Position),
                vec![2, 3]
            )),
            Instr::decode(&[6, 2, 3])
        );
        assert_eq!(
            Some(Instr(
                Opcode::LT(ParameterMode::Position, ParameterMode::Position),
                vec![2, 3, 4]
            )),
            Instr::decode(&[7, 2, 3, 4])
        );
        assert_eq!(
            Some(Instr(
                Opcode::EQ(ParameterMode::Position, ParameterMode::Position),
                vec![2, 3, 4]
            )),
            Instr::decode(&[8, 2, 3, 4])
        );
    }
}

#[cfg(test)]
mod parameter_mode_tests {

    use super::*;

    #[test]
    fn test_parse() {
        assert_eq!(Some(ParameterMode::Position), ParameterMode::parse(0));
        assert_eq!(Some(ParameterMode::Immediate), ParameterMode::parse(1));
        assert_eq!(None, ParameterMode::parse(2));
    }
}
