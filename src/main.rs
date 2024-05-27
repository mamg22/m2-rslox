use std::env;
use std::fs;
use std::io;
use std::process;

use m2_rslox::vm::InterpretResult;
use m2_rslox::vm::VM;

fn main() {
    let argv: Vec<String> = env::args().collect();

    let mut vm = VM::new();

    match argv.len() {
        1 => repl(&mut vm),
        2 => run_file(&mut vm, &argv[1]),
        _ => {
            eprintln!("Usage: {} [path]", argv[0]);
            process::exit(64);
        }
    }
}

fn repl(vm: &mut VM) {
    let stdin = io::stdin();
    let mut buf = String::new();

    loop {
        buf.clear();
        eprint!("> ");

        let bytes_read = stdin.read_line(&mut buf).unwrap();

        if bytes_read == 0 {
            eprintln!("");
            process::exit(0);
        }

        vm.interpret(&buf);
    }
}

fn run_file(vm: &mut VM, path: &str) {
    let source = fs::read_to_string(path).unwrap();

    let result: InterpretResult = vm.interpret(&source);

    let exit_code = match result {
        InterpretResult::CompileError => 65,
        InterpretResult::RuntimeError => 70,
        InterpretResult::Ok => 0,
    };
    process::exit(exit_code);
}