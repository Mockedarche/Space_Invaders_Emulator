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
