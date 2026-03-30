use crate::i8080::I8080Core;

mod i8080;

fn handle_out(port: u8, value: u8){
    if port == 0 {
        print!("{}", value as char);
    }
}


fn main() {
    let mut core = I8080Core::new();
    core.on_out = Some(handle_out);
    core.i8080_load_rom("Your Directory Here", 0x0100);
    println!("this is main");
}
