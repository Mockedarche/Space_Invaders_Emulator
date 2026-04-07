use i8080_emulator::{I8080Core, LoadRomResult, StepInstructionResult};
use std::{io::Write};


/*
 * handle_out - used to capture outs for output
 * Expects: core.on_out was correctly linked to this function as a Some
 * Does: Takes the out as expected for the CP/M for output
 * Returns: N/A
 */
fn handle_out(port: u8, value: u8) {
    let debug = false;
    if debug {
        println!("DID an OUT handle_out");
    }
    if port == 0 {
        if debug {
            println!("CHAR: {} (0x{:02X})", value as char, value);
        }
        print!("{}", value as char);
        std::io::stdout().flush().unwrap();
    }
    else if port == 1 {
        println!("*** FUNCTION 9 ENTERED ***");
    }
    else {
        if debug {
            println!("PORT IS :{}", port as u8);
        }
    }
}

fn run_test(s: &str) {
    // init core including the out port capture
    let mut core = I8080Core::new();
    core.on_out = Some(handle_out);

    // load the ROM into the core and print information related to how it went
    let load_rom_result = core.i8080_load_rom(s, 0x0100);
    match load_rom_result {
        LoadRomResult::Ok => {
            
        }
        LoadRomResult::NotFound => {
            println!("Failed to load ROM as it wasn't found");
        }
        LoadRomResult::Error => {
            println!("An Error occured with loading the rom");
        }
    }

    // Set up CP/M zero page
    core.memory[0x0000] = 0xC3; // JMP
    core.memory[0x0001] = 0x00;
    core.memory[0x0002] = 0x00;
    // 0x0005: JMP to BDOS handler at 0x0030
    core.memory[0x0005] = 0xC3;
    core.memory[0x0006] = 0x30;
    core.memory[0x0007] = 0x00;

    // Top of memory for stack initialization (LHLD 6 / SPHL)
    core.memory[0x0006] = 0x00;
    core.memory[0x0007] = 0xF0; 

    // BDOS handler is essentially the CP/M behavior that test ROMS expect
    let bdos_handler: [u8; 32] = [
        0x79,             
        0xFE, 0x02,       
        0xC2, 0x0A, 0xF0, 
        0x7B,             
        0xD3, 0x00,       
        0xC9,             
        0xFE, 0x09,       
        0xC2, 0x1F, 0xF0, 
        0x7B,             
        0x6F,             
        0x7A,             
        0x67,             
        0x7E,             
        0xFE, 0x24,       
        0xCA, 0x1F, 0xF0, 
        0xD3, 0x00,       
        0x23,             
        0xC3, 0x13, 0xF0, 
        0xC9,             
    ];
    // place the BDOS handler into the cores memory
    core.memory[0xF000..0xF000 + bdos_handler.len()].copy_from_slice(&bdos_handler);

    // init capture for the instruction execution result
    let mut step_result;
    // loop through the instructions in the rom printing information on failures or PC == 0 indicating finished
    loop {
        (step_result, _) = core.i8080_step();
        match step_result {
            StepInstructionResult::Halt => {
                println!("Encountered a HALT STOPPING");
                break;
            } 
            StepInstructionResult::Error => {
                println!("Step failed and returned an error so exiting");
                break;
            }
            StepInstructionResult::NoOperation =>{

            }
            StepInstructionResult::Ok => {
                if core.program_counter == 0 {
                    println!("\nHit PC 0");
                    break;
                }
            }
        }

        
    }
}

fn main() {
    let base = "Your rom directory here";
    let arr: [&str; 4] = ["TST8080.COM", "CPUTEST.COM", "8080PRE.COM", "8080EXM.COM"];


    for i in arr{
        println!("\n\n Running test {}", i);
        run_test(&format!("{}{}", base, i));
    }


}
