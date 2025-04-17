use qrlox_interpreter::world::InterpreterWorld;

pub struct CliWorld;

impl InterpreterWorld for CliWorld {
    fn println(&mut self, s: &str) {
        println!("{}", s);
    }
}
