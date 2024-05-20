use std::ops;

use crate::chunk::{Chunk, OpCode};
use crate::value::Value;
use crate::debug::disassemble_instruction;

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
}

impl VM {
    pub fn new(chunk: Chunk) -> Self {
        Self { chunk, ip: 0, stack: Vec::new() }
    }

    pub fn interpret(&mut self) -> InterpretResult {
        self.run()
    }

    pub fn run(&mut self) -> InterpretResult {
        loop {
            let instruction = &self.chunk.code()[self.ip];

            if cfg!(feature = "debug_trace_execution") {
                let stack_str: String = self.stack.iter()
                    .map(|elem| format!("[{}]", elem))
                    .collect();

                eprintln!("   Stack: {stack_str}");
                disassemble_instruction(&self.chunk, self.ip)
            }

            self.ip += 1;
            match instruction {
                OpCode::Return => {
                    eprintln!("{}", self.pop());
                    return InterpretResult::Ok
                },
                OpCode::Negate => {
                    let val = -self.pop();
                    self.push(val)
                },
                OpCode::Constant(id) => {
                    let const_val = self.read_constant(*id as usize);

                    self.push(const_val.clone());
                },
                OpCode::Add => self.binary_op(ops::Add::add),
                OpCode::Substract => self.binary_op(ops::Sub::sub),
                OpCode::Multiply => self.binary_op(ops::Mul::mul),
                OpCode::Divide => self.binary_op(ops::Div::div),
            }
        }
    }

    fn read_constant(&self, id: usize) -> &Value {
        &self.chunk.constants()[id]
    }

    fn binary_op(&mut self, op_func: fn(Value, Value) -> Value) {
        let b = self.pop();
        let a = self.pop();
        
        let result = op_func(a, b);
        self.push(result);
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }
}