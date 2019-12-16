use std::io::{BufRead, BufReader, Read};

fn main() {
    let f = std::fs::File::open(std::env::args().nth(1).expect("Could not get arg 1"))
        .expect("Could not open input file");

    let code: Vec<u32> = read_intcode(f).collect();

    for noun in 0..=99 {
        for verb in 0..=99 {
            let mut code = code.clone();
            code[1] = noun;
            code[2] = verb;

            let mut program = Program::new(code);

            while !program.finished {
                program.tick();
            }

            if program.code[0] == 19690720 {
                println!(
                    "noun = {}, verb = {}, 100 * noun + verb = {}",
                    noun,
                    verb,
                    100 * noun + verb
                );
                break;
            }
        }
    }
}

fn read_intcode<R>(read: R) -> impl Iterator<Item = u32>
where
    R: Read,
{
    BufReader::new(read)
        .split(b',')
        .map(|l| String::from_utf8(l.unwrap()).unwrap())
        .flat_map(|s| s.trim().parse::<u32>())
}

type Pc = usize;
type Code = Vec<u32>;

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
        let instr = decode_instr(&self.code[self.pc..]).unwrap();
        self.execute_instr(instr);
    }

    fn execute_instr(&mut self, instr: Instr) {
        use Instr::*;

        match instr {
            Add(a, b, t) => {
                self.code[t as usize] = self.code[a as usize] + self.code[b as usize];
                self.pc += 4;
            }
            Mul(a, b, t) => {
                self.code[t as usize] = self.code[a as usize] * self.code[b as usize];
                self.pc += 4;
            }
            Fin => self.finished = true,
        }
    }
}

#[derive(Debug, PartialEq)]
enum Opcode {
    Add,
    Mul,
    Fin,
}

#[derive(Debug, PartialEq)]
enum Instr {
    Add(u32, u32, u32),
    Mul(u32, u32, u32),
    Fin,
}

fn decode_opcode(i: u32) -> Option<Opcode> {
    match i {
        1 => Some(Opcode::Add),
        2 => Some(Opcode::Mul),
        99 => Some(Opcode::Fin),
        _ => None,
    }
}

fn decode_instr(code: &[u32]) -> Option<Instr> {
    match decode_opcode(code[0])? {
        Opcode::Add => Some(Instr::Add(code[1], code[2], code[3])),
        Opcode::Mul => Some(Instr::Mul(code[1], code[2], code[3])),
        Opcode::Fin => Some(Instr::Fin),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_decode_opcode() {
        assert_eq!(Some(Opcode::Add), decode_opcode(1));
        assert_eq!(Some(Opcode::Mul), decode_opcode(2));
        assert_eq!(Some(Opcode::Fin), decode_opcode(99));
        assert_eq!(None, decode_opcode(5));
    }

    #[test]
    fn test_decode_instr() {
        assert_eq!(Some(Instr::Add(2, 3, 4)), decode_instr(&[1, 2, 3, 4]));
        assert_eq!(Some(Instr::Mul(3, 4, 5)), decode_instr(&[2, 3, 4, 5]));
        assert_eq!(Some(Instr::Fin), decode_instr(&[99]));
    }

    #[test]
    fn test_execute_instr_add() {
        let mut program = Program::new(vec![0, 1, 2, 0]);

        program.execute_instr(Instr::Add(1, 2, 0));

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

        program.execute_instr(Instr::Mul(4, 5, 0));

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
    fn test_execute_instr_fin() {
        let mut program = Program::new(vec![99]);

        program.execute_instr(Instr::Fin);

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
}
