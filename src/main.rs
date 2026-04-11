use crate::i8080::{I8080Core};
use macroquad::prelude::*;

mod i8080;

/*
 * handle_out - Function
 * Expects: core to be a valid I8080Core, port to be the I/O port address, and value to be the byte to write
 * Does: Handles hardware output commands, specifically managing the bit-shift register data and offset
 */
fn handle_out(core: &mut I8080Core, port: u8, value: u8){
    if port == 0 {
        print!("{}", value as char);
    }
    else if port == 2 {
        core.shift_offset = value & 0x07;
    }
    else if port == 4 {
        core.shift_data = ((value as u16) << 8) | (core.shift_data >> 8);
    }
}

/*
 * handle_in - Function
 * Expects: core to be a valid I8080Core and port to be the requested I/O port address
 * Does: Handles hardware input requests, specifically calculating and returning the shifted byte from the shift register to the Accumulator
 */
fn handle_in(core: &mut I8080Core, port: u8){
    if port == 3 {
        let offset = 8 - (core.shift_offset as u16);
        core.a = (core.shift_data >> offset) as u8;
    }
}

/*
 * window_conf - Function
 * Expects: N/A
 * Does: Returns the configuration settings for the Macroquad window, including title and scaled dimensions
 */
fn window_conf() -> Conf {
    Conf {
        window_title: "Space Invaders".to_string(),
        window_width: (224.0 * 4.38) as i32,
        window_height: (256.0 * 4.38) as i32,
        high_dpi: false,
        ..Default::default()
    }
}

const VRAM_START: usize = 0x2400;
const SCREEN_WIDTH: usize = 224;
#[allow(dead_code)]
const SCREEN_HEIGHT: usize = 256;
const SCALE: f32 = 4.0;

/*
 * draw_framebuffer - Function
 * Expects: memory slice to contain valid Space Invaders VRAM at VRAM_START
 * Does: Iterates through VRAM and draws pixels to the screen, accounting for the 90-degree hardware rotation
 */
fn draw_framebuffer(memory: &[u8]) {
    for x in 0..SCREEN_WIDTH {
        for y_byte in 0..32 {
            let byte = memory[VRAM_START + (x * 32) + y_byte];
            
            for bit in 0..8 {
                if (byte >> bit) & 1 == 1 {
                    let y = (y_byte * 8 + bit) as f32;
                    let draw_x = x as f32;
                    let draw_y = 255.0 - y;

                    draw_rectangle(
                        draw_x * SCALE, 
                        draw_y * SCALE, 
                        SCALE, 
                        SCALE, 
                        WHITE
                    );
                }
            }
        }
    }
    draw_rectangle_lines(0.0, 0.0, 224.0 * SCALE, 256.0 * SCALE, 2.0, GREEN);
}

/*
 * main - Function (Async)
 * Expects: Space Invaders ROM files to be present at the specified paths
 * Does: Initializes the emulator core, loads ROMs, and runs the main execution loop including interrupt timing and rendering
 */
#[macroquad::main(window_conf)]
async fn main() {
    let mut core = I8080Core::new();
    core.on_out = Some(handle_out);
    core.on_in = Some(handle_in);
    core.i8080_load_rom("/home/mockedarche/Documents/Programming/Space_Invaders_Emulator/Space Invaders ROM/invaders.h", 0x0000);
    core.i8080_load_rom("/home/mockedarche/Documents/Programming/Space_Invaders_Emulator/Space Invaders ROM/invaders.g", 0x0800);
    core.i8080_load_rom("/home/mockedarche/Documents/Programming/Space_Invaders_Emulator/Space Invaders ROM/invaders.f", 0x1000);
    core.i8080_load_rom("/home/mockedarche/Documents/Programming/Space_Invaders_Emulator/Space Invaders ROM/invaders.e", 0x1800);
    core.set_program_counter_location(0x0000);
    println!("this is main");

    // Used to track timing 
    let t_states_per_frame: usize = 33333;
    let mut t_state_counter:usize = 0;

    // set background
    clear_background(BLACK);

    // used to track timing specifically for the frame operations
    let mut last_interrupt_t_states: usize = 0;
    let mut next_interrupt_num: u8 = 1;
    let t_states_per_half_frame = t_states_per_frame / 2; 

    loop {
        // perform a instruction
        let (_step_result, instruction_t_states) = core.i8080_step();
        t_state_counter += instruction_t_states as usize;

        // Check if its time to do a interrupt
        if t_state_counter - last_interrupt_t_states >= t_states_per_half_frame {
            if core.interrupt_enabled {
                if next_interrupt_num == 1 {
                    service_interrupt(&mut core, 0xCF); // RST 1
                    next_interrupt_num = 2;
                } else {
                    service_interrupt(&mut core, 0xD7); // RST 2
                    next_interrupt_num = 1;
                }
                core.interrupt_enabled = false; 
            }

            last_interrupt_t_states = t_state_counter;
        }

        // Check if its time to print a frame
        if t_state_counter >= t_states_per_frame {
            clear_background(BLACK);
            draw_framebuffer(&core.memory);

            t_state_counter -= t_states_per_frame;
            last_interrupt_t_states = 0;

            next_frame().await;
        }
    }
}

/*
 * service_interrupt - Function
 * Expects: core to be initialized and opcode to be a valid RST instruction (0xCF or 0xD7)
 * Does: Manually pushes the program counter to the stack and jumps to the appropriate interrupt vector
 */
fn service_interrupt(core: &mut I8080Core, opcode: u8) {
    // here we push program counter and perform the RST as requested from the interrupt
    let pc = core.program_counter;
    core.memory[(core.stack_pointer - 1) as usize] = ((pc >> 8) & 0xFF) as u8;
    core.memory[(core.stack_pointer - 2) as usize] = (pc & 0xFF) as u8;
    core.stack_pointer = core.stack_pointer.wrapping_sub(2);

    core.program_counter = match opcode {
        0xCF => 0x0008,
        0xD7 => 0x0010,
        _ => core.program_counter,
    };
}