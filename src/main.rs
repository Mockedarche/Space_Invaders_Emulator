/*
 * Austin Lunbeck's Implementation of the Space Invaders 1978 arcade game emulated in rust
 * Uses macroquad for audio, video, and input
 * 
 */
use crate::i8080::{I8080Core};
use macroquad::prelude::*;
use macroquad::audio::{load_sound, Sound, play_sound, stop_sound, PlaySoundParams};
use std::env;

mod i8080;

/*
 * SpaceInvaderSounds - Struct
 * Used to hold the Sound variables for an easy to access way to initialize and play each file
 * 
 */
struct SpaceInvadersSounds{
    ufo: Sound,
    shot: Sound,
    flash: Sound,
    invader_die: Sound,
    extended_play: Sound,
    fleet1: Sound,
    fleet2: Sound,
    fleet3: Sound,
    fleet4: Sound,
    ufo_hit: Sound,
}

/*
 * update_input - Function
 * Expects: core to be initialized
 * Does: Checks for keyboard input and sets the corresponding bits in the emulator's I/O ports
 */
fn update_input(core: &mut I8080Core) {

    // Used to store the bit representation of the player 1 input
    let mut p1_state: u8 = 0;

    // Left
    if is_key_down(KeyCode::Left)  { p1_state |= 1 << 5; }
    // Right
    if is_key_down(KeyCode::Right) { p1_state |= 1 << 6; }
    // Shoot
    if is_key_down(KeyCode::Space) { p1_state |= 1 << 4; }
    // Select player 1
    if is_key_pressed(KeyCode::Key1) { p1_state |= 1 << 2; }
    // Enter a coin 
    if is_key_pressed(KeyCode::C)    { p1_state |= 1 << 0; } 

    // store the player 1's action
    core.port1 = p1_state;
}

/*
 * handle_out - Function
 * Expects: core to be a valid I8080Core, port to be the I/O port address, and value to be the byte to write
 * Does: Handles hardware output commands, such as the shift register and sounds,
 */
fn handle_out(core: &mut I8080Core, port: u8, value: u8) {
    match port {
        0 => print!("{}", value as char),
        2 => core.shift_offset = value & 0x07,
        3 => {
            // Bit 0 UFO sound
            core.sound_triggers[0] = (value & 0x01) != 0;

            // Bit 1 Shot sound
            if (value & 0x02) != 0 {
                core.sound_triggers[1] = true;
            }
            // Bit 2 Player Die sound
            if (value & 0x04) != 0 {
                core.sound_triggers[2] = true;
            }
            // Bit 3 Invader Die sound
            if (value & 0x08) != 0 {
                core.sound_triggers[3] = true;
            }
            // Bit 4 Extended Play sound
            if (value & 0x10) != 0 {
                core.sound_triggers[4] = true;
            }
        }
        4 => core.shift_data = ((value as u16) << 8) | (core.shift_data >> 8),
        5 => {
            // Bit 0 Fast invader 1 sound
            if (value & 0x01) != 0 {
                core.sound_triggers[5] = true;
            }
            // Bit 1 Fast invader 2 sound
            if (value & 0x02) != 0 {
                core.sound_triggers[6] = true;
            }
            // Bit 2 Fast invader 3 sound
            if (value & 0x04) != 0 {
                core.sound_triggers[7] = true;
            }
            // Bit 3 Fast invader 4 sound
            if (value & 0x08) != 0 {
                core.sound_triggers[8] = true;
            }
            // Bit 4 Mystery ship (high pitch) / UFO Hit sound
            if (value & 0x10) != 0 {
                core.sound_triggers[9] = true;
            }
        }
        6 => (),
        _ => {
            println!("Got a port for handle out that isn't handled port: {}", port);
        }
    }
}

/*
 * handle_in - Function
 * Expects: core to be a valid I8080Core and port to be the requested I/O port address
 * Does: Handles hardware input requests, specifically calculating and returning the shifted byte from the shift register to the Accumulator
 */
fn handle_in(core: &mut I8080Core, port: u8){
    match port {
        1 => core.a = core.port1,
        2 => core.a = 0,
        3 => {
            let offset = 8 - (core.shift_offset as u16);
            core.a = (core.shift_data >> offset) as u8;
        }
        _ => {
            println!("Got a port for handle in that isn't handled port: {}", port);
        }
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

// Variables used to describe the space invaders 1978 screen
const VRAM_START: usize = 0x2400;
const SCREEN_WIDTH: usize = 224;
#[allow(dead_code)]
const SCREEN_HEIGHT: usize = 256;
const SCALE: f32 = 4.0;

/*
 * draw_framebuffer - Function
 * Expects: memory slice to contain valid Space Invaders VRAM at VRAM_START
 * Does: Iterates through VRAM and draws pixels to the screen, accounting for the 90-degree hardware rotation
 * using the colors as indicated from the palette
 */
fn draw_framebuffer(memory: &[u8], colors: (Color, Color, Color, Color)) {
    for x in 0..SCREEN_WIDTH {
        for y_byte in 0..32 {
            let byte = memory[VRAM_START + (x * 32) + y_byte];
            
            for bit in 0..8 {
                if (byte >> bit) & 1 == 1 {
                    let y = (y_byte * 8 + bit) as f32;
                    let draw_x = x as f32;
                    let draw_y = 255.0 - y;

                    let pixel_color = if draw_y > 32.0 && draw_y < 64.0 {
                        colors.0   // UFO Row
                    } else if draw_y > 183.0 && draw_y < 240.0 {
                        colors.1 // Main Green Band (Shields & Player)
                    } else if draw_y > 239.0 && draw_x > 20.0 && draw_x < 100.0 {
                        colors.1 // Bottom Ships (Lives) - specifically limiting X
                    } else {
                        colors.2 // Scores, Invaders, and Credit text
                    };

                    draw_rectangle(
                        draw_x * SCALE, 
                        draw_y * SCALE, 
                        SCALE, 
                        SCALE, 
                        pixel_color
                    );
                }
            }
        }
    }
    draw_rectangle_lines(0.0, 0.0, 224.0 * SCALE, 256.0 * SCALE, 2.0, colors.1);
}


/*
 * get_color_from_string - Function
 * Expects: N/A
 * Does: Returns the Color representation of the given string
 * Returns: The color representation of the given string
 * 
 */
fn get_color_from_string(color: &str) -> Color {
    match color.to_lowercase().as_str() {
        "lightgray"  => LIGHTGRAY,
        "gray"       => GRAY,
        "darkgray"   => DARKGRAY,
        "yellow"     => YELLOW,
        "gold"       => GOLD,
        "orange"     => ORANGE,
        "pink"       => PINK,
        "red"        => RED,
        "maroon"     => MAROON,
        "green"      => GREEN,
        "lime"       => LIME,
        "darkgreen"  => DARKGREEN,
        "skyblue"    => SKYBLUE,
        "blue"       => BLUE,
        "darkblue"   => DARKBLUE,
        "purple"     => PURPLE,
        "violet"     => VIOLET,
        "darkpurple" => DARKPURPLE,
        "beige"      => BEIGE,
        "brown"      => BROWN,
        "darkbrown"  => DARKBROWN,
        "white"      => WHITE,
        "black"      => BLACK,
        "blank"      => BLANK,
        "magenta"    => MAGENTA,
        _ => {
            println!("Color '{}' not found, defaulting to WHITE", color);
            WHITE
        }
    }
}

/*
 * get_random_palette - Function
 * Expects: N/A
 * Does: Returns a tuple of 4 different colors (no duplicates)
 * Returns: Tuple of 4 colors (no duplicates)
 * 
 */
fn get_random_palette() -> (Color, Color, Color, Color) {
    // 1. Every named color constant in Macroquad
    let mut pool = vec![
        LIGHTGRAY, GRAY, DARKGRAY, YELLOW, GOLD, ORANGE, PINK, RED, MAROON,
        GREEN, LIME, DARKGREEN, SKYBLUE, BLUE, DARKBLUE, PURPLE, VIOLET,
        DARKPURPLE, BEIGE, BROWN, DARKBROWN, WHITE, MAGENTA
    ];

    // 2. Fisher-Yates shuffle (using Macroquad's built-in rand)
    // We only need to shuffle the first 3 positions to get our results
    for i in 0..4 {
        let j = rand::gen_range(i, pool.len());
        pool.swap(i, j);
    }

    // 3. Return the first three unique results
    (pool[0], pool[1], pool[2], pool[3])
}


/*
 * main - Function (Async)
 * Expects: Space Invaders ROM files to be present at the specified paths
 * Does: Initializes the emulator core, loads ROMs, and runs the main execution loop including interrupt timing and rendering
 */
#[macroquad::main(window_conf)]
async fn main() {

    // Get any arguments
    let args: Vec<String> = env::args().collect();

    // Default palette
    let mut palette = (RED, GREEN, WHITE, BLACK);

    // Seed the random number generator using the system clock
    rand::srand(miniquad::date::now() as u64);

    // Check arguements
    // If we have 5 we assume user wants to give specific color palette
    if args.len() == 5 {
        palette = (get_color_from_string(&args[1]), get_color_from_string(&args[2]), get_color_from_string(&args[3]), get_color_from_string(&args[4]));
    } else if args.len() == 2 {
        // -random_colors makes the palette randomized once
        if args[1] == "-random_colors"{ 
            palette = get_random_palette();
        // -rainbow_mode is a flag that changes it every 8 seconds
        } else if args[1] == "-rainbow_mode" {
            palette = get_random_palette();
        }
    } else {
        println!("No arguments provided.");
    }

    // Init the core and load the rom
    let mut core = I8080Core::new();
    core.on_out = Some(handle_out);
    core.on_in = Some(handle_in);
   // Expected Folders
    let rom_dir = "Space Invaders ROM";
    let audio_dir = "audio";

    // Load ROMs
    core.i8080_load_rom(&format!("{}/invaders.h", rom_dir), 0x0000);
    core.i8080_load_rom(&format!("{}/invaders.g", rom_dir), 0x0800);
    core.i8080_load_rom(&format!("{}/invaders.f", rom_dir), 0x1000);
    core.i8080_load_rom(&format!("{}/invaders.e", rom_dir), 0x1800);
    core.set_program_counter_location(0x0000);

    // Load all the sound files
    let sounds = SpaceInvadersSounds {
        ufo:           load_sound(&format!("{}/ufo_lowpitch.wav", audio_dir)).await.expect("Failed to load ufo sound"),
        shot:          load_sound(&format!("{}/shoot.wav", audio_dir)).await.expect("Failed to load shoot sound"),
        flash:         load_sound(&format!("{}/explosion.wav", audio_dir)).await.expect("Failed to load explosion sound"),
        invader_die:   load_sound(&format!("{}/invaderkilled.wav", audio_dir)).await.expect("Failed to load invader die sound"),
        extended_play: load_sound(&format!("{}/invaderkilled.wav", audio_dir)).await.expect("Failed to load invader die sound"),
        fleet1:        load_sound(&format!("{}/fastinvader1.wav", audio_dir)).await.expect("Failed to load fleet 1 sound"),
        fleet2:        load_sound(&format!("{}/fastinvader2.wav", audio_dir)).await.expect("Failed to load fleet 2 sound"),
        fleet3:        load_sound(&format!("{}/fastinvader3.wav", audio_dir)).await.expect("Failed to load fleet 3 sound"),
        fleet4:        load_sound(&format!("{}/fastinvader4.wav", audio_dir)).await.expect("Failed to load fleet 4 sound"),
        ufo_hit:       load_sound(&format!("{}/ufo_highpitch.wav", audio_dir)).await.expect("Failed to load ufo high pitch sound"),
    };
    // Used to handle the looping sound (unique sound)
    let mut ufo_audio_playing = false;

    
    //Block execution until the user presses enter in the console. 
    println!("ROMs loaded. Press ENTER in the terminal to start the emulator...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    // Used to track timing 
    let t_states_per_frame: usize = 33333;
    let mut t_state_counter:usize = 0;

    // Set background
    clear_background(palette.3);

    // Used to track timing specifically for the frame operations
    let mut last_interrupt_t_states: usize = 0;
    let mut next_interrupt_num: u8 = 1;
    let t_states_per_half_frame = t_states_per_frame / 2; 
    // Used for rainbow mode
    let mut frame_count = 1;

    loop {
        // Perform a instruction
        let (_step_result, instruction_t_states) = core.i8080_step();
        t_state_counter += instruction_t_states as usize;

        // Check if its time to do a interrupt
        if t_state_counter - last_interrupt_t_states >= t_states_per_half_frame {
            if core.interrupt_enabled {
                if next_interrupt_num == 1 {
                    // RST 1
                    service_interrupt(&mut core, 0xCF); 
                    next_interrupt_num = 2;
                } else {
                    // RST 2
                    service_interrupt(&mut core, 0xD7); 
                    next_interrupt_num = 1;
                }
                core.interrupt_enabled = false; 
            }

            last_interrupt_t_states = t_state_counter;
        }

        // Check if its time to print a frame, do a sound, or change palette
        if t_state_counter >= t_states_per_frame {
            // Update the emulators view of the keyboard
            update_input(&mut core);

            if core.sound_triggers[0] && !ufo_audio_playing {
                play_sound(&sounds.ufo, PlaySoundParams { looped: true, volume: 0.6 });
                ufo_audio_playing = true;
            } else if !core.sound_triggers[0] && ufo_audio_playing {
                stop_sound(&sounds.ufo);
                ufo_audio_playing = false;
            }

            // Event specific sounds
            if core.sound_triggers[1] { 
                play_sound(&sounds.shot, PlaySoundParams { looped: false, volume: 0.08 });
                core.sound_triggers[1] = false; 
            }
            if core.sound_triggers[2] { 
                play_sound(&sounds.flash, PlaySoundParams { looped: false, volume: 0.08 });
                core.sound_triggers[2] = false; 
            }
            if core.sound_triggers[3] { 
                play_sound(&sounds.invader_die, PlaySoundParams { looped: false, volume: 0.05 });
                core.sound_triggers[3] = false; 
            }
            if core.sound_triggers[4] { 
                play_sound(&sounds.extended_play, PlaySoundParams { looped: false, volume: 0.05 });
                core.sound_triggers[4] = false; 
            }

            // Heartbeat of the game (invaders movement)
            if core.sound_triggers[5] { 
                play_sound(&sounds.fleet1, PlaySoundParams { looped: false, volume: 0.7 });
                core.sound_triggers[5] = false; 
            }
            if core.sound_triggers[6] { 
                play_sound(&sounds.fleet2, PlaySoundParams { looped: false, volume: 0.7 });
                core.sound_triggers[6] = false; 
            }
            if core.sound_triggers[7] { 
                play_sound(&sounds.fleet3, PlaySoundParams { looped: false, volume: 0.7 });
                core.sound_triggers[7] = false; 
            }
            if core.sound_triggers[8] { 
                play_sound(&sounds.fleet4, PlaySoundParams { looped: false, volume: 0.7 });
                core.sound_triggers[8] = false; 
            }
            if core.sound_triggers[9] { 
                play_sound(&sounds.ufo_hit, PlaySoundParams { looped: false, volume: 0.15 });
                core.sound_triggers[9] = false; 
            }

            // If its rainbow mode AND time to change the palette we randomize it
            if args.len() > 2 && args[1] == "-rainbow_mode" && frame_count % 480 == 0{
                palette = get_random_palette();
                frame_count = 1;
            }

            // Clear the background for a new frame
            clear_background(palette.3);
            draw_framebuffer(&core.memory, palette);

            // Track frame
            frame_count += 1;

            // Reset cycle count
            t_state_counter -= t_states_per_frame;
            last_interrupt_t_states = 0;

            // Set up to wait for next frame
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