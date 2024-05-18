use crate::chunk::{OpCode, Chunk};

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    eprintln!("== {name} ==");

    for (offset, instruction) in chunk.code().iter().enumerate() {
        eprint!("{offset:04} ");

        let current_line = chunk.lines()[offset];
        if offset > 0 && current_line == chunk.lines()[offset - 1] {
            eprint!("   | ");
        }
        else {
            eprint!("{:4} ", current_line);
        }

        match instruction {
            OpCode::Return => eprintln!("{:?}", instruction),
            OpCode::Constant(id) => {
                let val = chunk.constants()[*id as usize];
                eprintln!("{:?} {:?}", instruction, val);
            },
        };
    }
}