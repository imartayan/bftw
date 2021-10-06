use std::convert::TryFrom;
use std::env;
use std::fs;
use std::io::Read;
use std::str::Chars;

struct Program(Vec<Instr>);

enum Instr {
    Right,
    Left,
    Incr,
    Decr,
    Output,
    Input,
    Block(Program),
}

struct VM {
    data: Vec<u8>,
    cursor: usize,
}

#[derive(Debug)]
enum CompileError {
    MissingBracket,
    ExcessiveBracket,
}

#[derive(Debug)]
enum RuntimeError {
    CannotMoveLeft,
}

impl Program {
    fn parse(input: &mut impl Iterator<Item = char>, block: bool) -> Result<Self, CompileError> {
        let mut program: Vec<Instr> = Vec::new();
        let mut next: Option<char> = input.next();
        while next != None {
            let instr = match next.unwrap() {
                '>' => Instr::Right,
                '<' => Instr::Left,
                '+' => Instr::Incr,
                '-' => Instr::Decr,
                '.' => Instr::Output,
                ',' => Instr::Input,
                '[' => Instr::Block(Program::parse(input, true)?),
                ']' => {
                    return block
                        .then(|| ())
                        .map(|_| Program(program))
                        .ok_or(CompileError::ExcessiveBracket)
                }
                _ => {
                    next = input.next();
                    continue;
                }
            };
            program.push(instr);
            next = input.next();
        }
        (!block)
            .then(|| ())
            .map(|_| Program(program))
            .ok_or(CompileError::MissingBracket)
    }
}

impl TryFrom<String> for Program {
    type Error = CompileError;
    fn try_from(source: String) -> Result<Self, Self::Error> {
        let mut input: Chars = source.chars();
        Program::parse(&mut input, false)
    }
}

impl VM {
    fn new() -> Self {
        VM {
            data: vec![0],
            cursor: 0,
        }
    }

    fn execute(&mut self, Program(v): &Program) -> Result<(), RuntimeError> {
        let mut buffer: [u8; 1] = [0];
        for instr in v {
            match instr {
                Instr::Right => {
                    self.cursor += 1;
                    if self.cursor >= self.data.len() {
                        self.data.push(0)
                    }
                }
                Instr::Left => {
                    if self.cursor > 0 {
                        self.cursor -= 1
                    } else {
                        return Err(RuntimeError::CannotMoveLeft);
                    }
                }
                Instr::Incr => {
                    if self.data[self.cursor] < 255 {
                        self.data[self.cursor] += 1
                    } else {
                        self.data[self.cursor] = 0
                    }
                }
                Instr::Decr => {
                    if self.data[self.cursor] > 0 {
                        self.data[self.cursor] -= 1
                    } else {
                        self.data[self.cursor] = 255
                    }
                }
                Instr::Output => print!("{}", self.data[self.cursor] as char),
                Instr::Input => {
                    std::io::stdin()
                        .read_exact(&mut buffer)
                        .expect("Cannot read input");
                    self.data[self.cursor] = buffer[0]
                }
                Instr::Block(p) => {
                    while self.data[self.cursor] != 0 {
                        self.execute(&p)?
                    }
                }
            }
        }
        Ok(())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let source: String = if args.len() > 1 {
        let filename = &args[1];
        fs::read_to_string(filename).expect("Cannot read file")
    } else {
        String::from("++++++++[>+>++++++>++++<<<-]>[>+.>.<<-]")
    };
    let program: Program = Program::try_from(source).expect("Compilation error");
    let mut vm = VM::new();
    vm.execute(&program).expect("Runtime error")
}
