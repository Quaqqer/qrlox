use pretty_assertions::assert_str_eq;
use qrlox_compiler::resolve_program;
use qrlox_interpreter::{world::InterpreterWorld, Interpreter};
use qrlox_syntax::parse_program;

pub struct TestWorld {
    output: String,
}

impl InterpreterWorld for TestWorld {
    fn println(&mut self, s: &str) {
        self.output += s;
        self.output += "\n";
    }
}

pub fn test_io(program_source: &str, expected_output: &str) {
    let ariadne_config = ariadne::Config::new();
    let cache = ariadne::Source::from(program_source);

    let (program, errors) = parse_program(program_source, &ariadne_config);

    let Some(program) = program else {
        for err in errors {
            err.eprint(cache.clone()).unwrap();
        }

        panic!();
    };

    let resolved = match resolve_program(&program, &ariadne_config) {
        Ok(v) => v,
        Err(err) => {
            err.eprint(cache.clone()).unwrap();
            panic!();
        }
    };

    let world = TestWorld {
        output: String::new(),
    };
    let mut interpreter = Interpreter::new(world);

    let result = interpreter.exec_program(&resolved, &ariadne_config);

    match result {
        Ok(_) => {}
        Err(err) => {
            err.eprint(cache.clone()).unwrap();
            panic!();
        }
    }

    assert_str_eq!(
        interpreter.ctx.world.output.trim(),
        expected_output.trim(),
        "testing that output matches expected output"
    );
}
