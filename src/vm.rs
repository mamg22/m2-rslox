use std::ops;

use crate::chunk::{Chunk, OpCode};
use crate::compiler::Compiler;
use crate::value::Value;
use crate::debug::disassemble_instruction;

pub enum InterpretResult {
    CompileError,
    RuntimeError,
}

pub struct VM {
    chunk: Option<Chunk>,
    ip: usize,
    stack: Vec<Value>,
}

impl VM {
    pub fn new() -> Self {
        Self { chunk: None, ip: 0, stack: Vec::new() }
    }

    pub fn interpret(&mut self, source: &str) -> Result<(), InterpretResult> {
        let mut compiler = Compiler::new(source);

        match compiler.compile() {
            Ok(chunk) => {
                self.chunk = Some(chunk);
                self.ip = 0;
            },
            Err(_) => return Err(InterpretResult::CompileError),
        }

        self.run()
    }

    pub fn run(&mut self) -> Result<(), InterpretResult> {
        if self.chunk().code().len() == 0 {
            return Ok(());
        }
        loop {
            let ip = self.ip;
            self.ip += 1;
            let instruction: &OpCode = &self.chunk().code()[ip];

            if cfg!(feature = "debug_trace_execution") {
                let stack_str: String = self.stack.iter()
                    .map(|elem| format!("[{}]", elem))
                    .collect();

                eprintln!("   Stack: {stack_str}");
                disassemble_instruction(&self.chunk(), ip)
            }

            match instruction {
                OpCode::Return => {
                    eprintln!("{}", self.pop());
                    return Ok(())
                },
                OpCode::Negate => {
                    if let Value::Number(_) = self.peek(0) {
                        let val = self.pop();
                        self.push(-val);
                    }
                    else {
                        self.runtime_error("Operand must be a number");
                        return Err(InterpretResult::RuntimeError);
                    }
                },
                OpCode::Constant(id) => {
                    let const_val = self.read_constant(*id as usize);

                    self.push(const_val.clone());
                },
                OpCode::Nil => self.push(Value::Nil),
                OpCode::True => self.push(Value::Bool(true)),
                OpCode::False => self.push(Value::Bool(false)),
                OpCode::Add => self.binary_op(ops::Add::add)?,
                OpCode::Substract => self.binary_op(ops::Sub::sub)?,
                OpCode::Multiply => self.binary_op(ops::Mul::mul)?,
                OpCode::Divide => self.binary_op(ops::Div::div)?,
            }
        }
    }

    fn reset_stack(&mut self) {
        self.stack.clear();
    }

    fn runtime_error(&mut self, message: &str) {
        eprintln!("{}", message);
        eprintln!("[line {}] in script", self.chunk.as_ref().unwrap().lines()[self.ip - 1]);
        self.reset_stack();
    }

    fn read_constant(&self, id: usize) -> &Value {
        &self.chunk().constants()[id]
    }

    fn binary_op(&mut self, op_func: fn(Value, Value) -> Value) -> Result<(), InterpretResult> {
        match (self.peek(0), self.peek(1)) {
            (Value::Number(_), Value::Number(_)) => {
                let b = self.pop();
                let a = self.pop();
                
                let result = op_func(a, b);
                self.push(result);
                Ok(())
            },
            _ => {
                self.runtime_error("Opernds must be numbers");
                Err(InterpretResult::RuntimeError)
            }
        }
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn peek(&self, distance: usize) -> &Value {
        self.stack.iter().rev().nth(distance).unwrap()
    }

    fn chunk(&self) -> &Chunk {
        self.chunk.as_ref().expect("No chunk loaded in VM")
    }
}