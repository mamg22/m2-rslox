use crate::value::Value;

#[derive(Debug)]
pub enum OpCode {
    Constant(u8),
    Nil,
    True,
    False,
    Add,
    Substract,
    Multiply,
    Divide,
    Negate,
    Return,
}

pub struct Chunk {
    code: Vec<OpCode>,
    lines: Vec<usize>,
    constants: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            code: Vec::new(),
            lines: Vec::new(),
            constants: Vec::new(),
        }
    }

    pub fn code(&self) -> &Vec<OpCode> {
        &self.code
    }

    pub fn constants(&self) -> &Vec<Value> {
        &self.constants
    }

    pub fn lines(&self) -> &Vec<usize> {
        &self.lines
    }

    pub fn write(&mut self, op: OpCode, line: usize) {
        self.code.push(op);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> u8 {
        self.constants.push(value);
        (self.constants.len() - 1).try_into().unwrap()
    }
}