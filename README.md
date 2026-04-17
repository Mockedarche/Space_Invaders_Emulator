Space for my Space Invaders (1978) emulator implementation. Using my Intel 8080 emulator core
[I8080 Core](https://github.com/Mockedarche/Intel-8080-Emulator)

Completely functioning
![](https://github.com/Mockedarche/Space_Invaders_Emulator/blob/main/Media/default.gif)

Ability to change the colors as desired or at random
![](https://github.com/Mockedarche/Space_Invaders_Emulator/blob/main/Media/color_change.gif)

Notes
All written in rust using macroquad for audio, video, and input

Change the colors as desired with arguments
examples
```
cargo run -- purple green brown black

cargo run -- -random_colors

cargo run -- -rainbow_mode
```
List of accepted colors
lightgray, gray, darkgray, yellow, gold, orange, pink, red, maroon,
green, lime, darkgreen, skyblue, blue, darkblue, purple, violet,
darkpurple, beige, brown, darkbrown, white, magenta

ROM and audio files expected to be placed in the audio folder and rom folder respectively. This segment of code shows the filenames expected (which from what i've found are all the same but still)
```
    // Load ROMs
    core.i8080_load_rom(&format!("{}/invaders.h", rom_dir), 0x0000);
    core.i8080_load_rom(&format!("{}/invaders.g", rom_dir), 0x0800);
    core.i8080_load_rom(&format!("{}/invaders.f", rom_dir), 0x1000);
    core.i8080_load_rom(&format!("{}/invaders.e", rom_dir), 0x1800);

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
```
