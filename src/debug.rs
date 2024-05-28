use crate::chunk::{OpCode, Chunk};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    eprintln!("== {name} ==");

    for (offset, _) in chunk.code().iter().enumerate() {
        disassemble_instruction(chunk, offset);
    }
}

pub fn disassemble_instruction(chunk: &Chunk, offset: usize) {
    let instruction = &chunk.code()[offset];
    eprint!("{offset:04} ");

    let current_line = chunk.lines()[offset];
    if offset > 0 && current_line == chunk.lines()[offset - 1] {
        eprint!("   | ");
    }
    else {
        eprint!("{:4} ", current_line);
    }

    match instruction {
        OpCode::Return | OpCode::Negate |
        OpCode::Add | OpCode::Substract |
        OpCode::Multiply | OpCode::Divide |
        OpCode::Nil | OpCode::True | OpCode::False
            => eprintln!("{:?}", instruction),
        OpCode::Constant(id) => {
            let val = &chunk.constants()[*id as usize];
            eprintln!("{:?} {:?}", instruction, val);
        },
    };
}