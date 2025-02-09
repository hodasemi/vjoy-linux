# VJoy Linux

## What is this?
This is a tool to create a mapping from one or multiple source input devices to one or more output devices. I created this tool because I could not find anything that does it in a generic enough way that pleased me.

I tried to fix that X4 on linux does not correctly recognize the Thrustmaster T.16000M dual sticks. Both sticks report the exact same VID:PID and map layout, thus the game only lists one of them.

## Usage

Creating a mapping that takes both Thrustmaster joysticks as they are and create a virtual device that passes through every input from the source joystick.

```bash
vjoy-linux -i "Thrustmaster T.16000M,Thrustmaster T.16000M" -d "Joystick 1,Joystick 2" -o example_file.ron
```

The result is a file in ron-format (**R**ust **O**bject **N**otation) with all mapping information required. (There are examples in the `example_descriptor` directory)

To actually run a description just execute:

```bash
vjoy-linux -f example_file.ron
```

This takes the created file, creates both defined virtual devices and passes the input into them.

## How to build

```Bash
cargo build --release
```