use crate::i8080::I8080Core;
use macroquad::prelude::*;

mod i8080;

fn handle_out(port: u8, value: u8){
    if port == 0 {
        print!("{}", value as char);
    }
}

const VRAM_START: usize = 0x2400;
const SCREEN_WIDTH: usize = 224;
const SCREEN_HEIGHT: usize = 256;
const SCALE: f32 = 3.0; // make pixels visible

fn draw_framebuffer(memory: &[u8]) {
    for i in 0..(SCREEN_WIDTH * SCREEN_HEIGHT / 8) {
        let byte = memory[VRAM_START + i];

        for bit in 0..8 {
            if (byte >> bit) & 1 == 1 {
                let pixel_index = i * 8 + bit;

                // original layout is rotated
                let x = pixel_index / SCREEN_HEIGHT;
                let y = pixel_index % SCREEN_HEIGHT;

                // rotate to normal orientation
                let draw_x = y as f32;
                let draw_y = (SCREEN_WIDTH as i32 - x as i32) as f32;

                draw_rectangle(
                    draw_x * SCALE,
                    draw_y * SCALE,
                    SCALE,
                    SCALE,
                    WHITE,
                );
            }
        }
    }
}


/*
fn main() {
    /*
    let mut core = I8080Core::new();
    core.on_out = Some(handle_out);
    core.i8080_load_rom("Your Directory Here", 0x0100);
    println!("this is main");
    */
}
*/
#[macroquad::main("Space Invaders Emulator")]
async fn main() {
    let mut core = I8080Core::new();
    core.on_out = Some(handle_out);
    core.i8080_load_rom("Your rom directory here", 0x0000);
    core.i8080_load_rom("Your rom directory here", 0x0800);
    core.i8080_load_rom("Your rom directory here", 0x1000);
    core.i8080_load_rom("Your rom directory here", 0x1800);
    core.set_program_counter_location(0x0000);
    println!("this is main");

    loop {
        clear_background(BLACK);

        draw_framebuffer(&core.memory);

        next_frame().await;
    }
}