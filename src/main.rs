use clap::{arg, command, value_parser, ArgAction};
use display::Screen;
use memory::Memory;
use rand::random;
use sfml::system::{sleep, Clock, Time};
use std::fs;

mod display;
mod memory;

fn main() -> Result<(), String> {
    let matches = command!()
        .arg(arg!(path: [path] "path to the rom file").required(true))
        .arg(
            arg!(-l --legacy ... "run with old instructions on")
                .required(false)
                .action(ArgAction::SetTrue),
        )
        .arg(
            arg!(-f --frequency [FREQUENCY] ... "run with specified frequency")
                .required(false)
                .value_parser(value_parser!(f32))
                .default_value("700"),
        )
        .get_matches();
    let mut screen = Screen::new((800u32, 400u32), "chip-8");
    let mut memory = Memory::new();
    let mut cycle_clock = Clock::start();
    let old_instructions = matches.get_flag("legacy");
    let frequency = matches.get_one::<f32>("frequency").unwrap();

    load_rom(&mut memory, matches.get_one::<String>("path").unwrap())?;

    loop {
        screen.handle_events();
        if screen.closed() {
            break;
        }
        emulate_cycle(&mut memory, &mut screen, old_instructions)?;
        update_timers(&mut cycle_clock, &mut memory);
        sleep(Time::seconds(1f32 / frequency));
    }
    Ok(())
}

fn emulate_cycle(
    memory: &mut Memory,
    screen: &mut Screen,
    old_instructions: bool,
) -> Result<(), String> {
    let op_code = fetch(memory);
    decode_and_execute(op_code, screen, memory, old_instructions)
}

fn update_timers(cycle_clock: &mut Clock, memory: &mut Memory) {
    if cycle_clock.elapsed_time().as_seconds() > 1f32 / 60f32 {
        memory.decrement_delay();
        memory.decrement_sound();
    } else {
        cycle_clock.restart();
    }
}

fn fetch(memory: &mut Memory) -> u16 {
    let ins_mem = memory.fetch_instruction();
    memory.increment_pc();
    (ins_mem.0 as u16) << 8 | (ins_mem.1 as u16)
}

fn decode_and_execute(
    op_code: u16,
    screen: &mut Screen,
    memory: &mut Memory,
    old_instructions: bool,
) -> Result<(), String> {
    let x = ((0x0F00u16 & op_code) >> 8) as u8;
    let y = ((0x00F0u16 & op_code) >> 4) as u8;
    let n = (0x000Fu16 & op_code) as u8;
    let nn = (0x00FFu16 & op_code) as u8;
    let nnn = 0x0FFFu16 & op_code;

    let res = match op_code & 0xF000 {
        0x0000u16 => zero_instructions(op_code, screen, memory),
        0x1000u16 => Ok(memory.jump_pc(nnn)), // 1NNN: jump
        0x2000u16 => Ok(call_subroutine(nnn, memory)), // 2NNN: call subroutine
        0x3000u16 => skip_if_eq_im(x, nn, memory), // 3XNN: skip if var[x] == nn
        0x4000u16 => skip_if_neq_im(x, nn, memory), // 4XNN: skip if var[x] != nn
        0x5000u16 => skip_if_eq(x, y, memory), // 5XY0: skip if var[x] == var[y]
        0x6000u16 => memory.set_var_register(x, nn), //6XNN: var[x] := nn
        0x7000u16 => add_var_register(memory, x, nn), // 7XNN: var[x] := var[x] + nn
        0x8000u16 => basic_operations(op_code, memory, old_instructions),
        0x9000u16 => skip_if_neq(x, y, memory), // 9XY0: skip if var[x] != var[y]
        0xA000u16 => Ok(memory.set_index_register(nnn)), // ANNN: I := nnn
        0xB000u16 => jump_with_offset(memory, x, nnn, old_instructions), // BXNN: jump with offset
        0xC000u16 => memory.set_var_register(x, random::<u8>() & nn), // CXNN: V[x] := rand & nn
        0xD000u16 => draw_sprite(x, y, n, memory, screen), // DXYN: Display (Draw)
        0xE000u16 => skip_if_key(op_code, x, memory, screen),
        0xF000u16 => f_instructions(op_code, x, memory, screen, old_instructions),
        _ => Err("Invalid op code".to_string()),
    };
    if let Err(err) = res {
        return Err(format!(
            "Error in instrcution with opcode {:#06x}: \n {}",
            op_code, err
        ));
    }
    res
}

fn f_instructions(
    op_code: u16,
    x: u8,
    memory: &mut Memory,
    screen: &mut Screen,
    old_instructions: bool,
) -> Result<(), String> {
    let vx = memory.get_var_register(x)?;
    let delay_timer = memory.delay_register();
    match op_code & 0x00FF {
        0x0007 => memory.set_var_register(x, delay_timer)?, // FX07: var[x] := delay_timer
        0x0015 => memory.set_delay_register(vx),            // FX15: delay_timer := var[x]
        0x0018 => memory.set_sounds_register(vx),           // FX18: sound_timer := var[x]
        0x001E => add_to_index(memory, vx)?,                // FX1E: I := I + var[x]
        0x000A => wait_for_keyinput(memory, screen, x)?,    // FX0A: get key input
        0x0029 => get_font_char(memory, vx), // FX29: I := Font offset of font char var[x]
        0x0033 => to_digits(memory, vx),     // FX33: 623 -> 6, 2, 3
        0x0055 => store_registers(memory, x, old_instructions)?, // FX55: store registers in ram
        0x0065 => load_registers(memory, x, old_instructions)?, // FX65: load registers from ram
        _ => return Err("Invalid op code".to_string()),
    };
    Ok(())
}

fn load_registers(memory: &mut Memory, x: u8, old_instructions: bool) -> Result<(), String> {
    let index = memory.index_register();
    let mut register;
    for i in 0..=x {
        register = memory.read_ram_cell(index + i as u16);
        memory.set_var_register(i, register)?;
    }
    if old_instructions {
        memory.set_index_register(index + x as u16 + 1u16);
    }
    Ok(())
}

fn store_registers(memory: &mut Memory, x: u8, old_instructions: bool) -> Result<(), String> {
    let index = memory.index_register();
    let mut register_buffer = Vec::with_capacity(x as usize + 1);
    for i in 0..=x {
        register_buffer.push(memory.get_var_register(i)?);
    }
    memory.write_ram(index, &register_buffer);
    if old_instructions {
        memory.set_index_register(index + x as u16 + 1u16);
    }
    Ok(())
}

fn to_digits(memory: &mut Memory, mut vx: u8) {
    let index_register = memory.index_register();
    let mut digits: [u8; 3] = [0; 3];
    for digit in digits.iter_mut().rev() {
        *digit = vx % 10;
        vx /= 10;
    }
    memory.write_ram(index_register, &digits);
}

fn get_font_char(memory: &mut Memory, vx: u8) {
    let char = vx & 0x0F;
    memory.set_index_register(0x0050 + 5 * char as u16)
}

fn wait_for_keyinput(memory: &mut Memory, screen: &mut Screen, x: u8) -> Result<(), String> {
    if screen.any_key_pressed() {
        memory.set_var_register(x, screen.get_pressed_key())?
    } else {
        memory.decrement_pc();
    }
    Ok(())
}

fn add_to_index(memory: &mut Memory, vx: u8) -> Result<(), String> {
    let res = memory.index_register() as usize + vx as usize;
    if res > 0x0FFF {
        memory.set_var_register(0xF, 1)?;
    }
    memory.set_index_register(res as u16);
    Ok(())
}

fn skip_if_key(opcode: u16, x: u8, memory: &mut Memory, screen: &mut Screen) -> Result<(), String> {
    let is_pressed = screen.key_state(memory.get_var_register(x)?)?;
    match opcode & 0x00FF {
        0x009E => {
            if is_pressed {
                memory.increment_pc()
            }
        }
        0x00A1 => {
            if !is_pressed {
                memory.increment_pc()
            }
        }
        _ => return Err("Invalid op code".to_string()),
    }
    Ok(())
}

fn jump_with_offset(
    memory: &mut Memory,
    x: u8,
    nnn: u16,
    old_instructions: bool,
) -> Result<(), String> {
    if old_instructions {
        let v0 = memory.get_var_register(0).unwrap();
        memory.jump_pc(nnn + v0 as u16);
    } else {
        let vx = memory.get_var_register(x)?;
        memory.jump_pc(nnn + vx as u16);
    }
    Ok(())
}

fn basic_operations(
    op_code: u16,
    memory: &mut Memory,
    old_instructions: bool,
) -> Result<(), String> {
    let x = ((0x0F00u16 & op_code) >> 8) as u8;
    let y = ((0x00F0u16 & op_code) >> 4) as u8;
    let vx = memory.get_var_register(x)?;
    let vy = memory.get_var_register(y)?;

    match op_code & 0x000Fu16 {
        0x0000u16 => memory.set_var_register(x, vy), // 8XY0: var[x] := var[y]
        0x0001u16 => memory.set_var_register(x, vx | vy), // 8XY1: var[x] := var[y] | var[x]
        0x0002u16 => memory.set_var_register(x, vx & vy), // 8XY2: var[x] := var[y] & var[x]
        0x0003u16 => memory.set_var_register(x, vx ^ vy), // 8XY3: var[x] := var[x] ^ var[y]
        0x0004u16 => add(memory, x, vx, vy),         // 8XY4: var[x] := var[x] + var[y]
        0x0005u16 => sub_x_y(memory, x, vx, vy),     // 8XY5: var[x] := var[x] - var[y]
        0x0006u16 => shift_right(memory, x, vx, vy, old_instructions), // 8XY6: var[x] := var[x] >> 1
        0x0007u16 => sub_y_x(memory, x, vx, vy), // 8XY7: var[x] := var[y] - var[x]
        0x000Eu16 => shift_left(memory, x, vx, vy, old_instructions), // 8XYE: var[x] := var[x] << 1
        _ => Err("Invalid op code".to_string()),
    }
}

fn shift_right(
    memory: &mut Memory,
    x: u8,
    vx: u8,
    vy: u8,
    old_instructions: bool,
) -> Result<(), String> {
    if old_instructions {
        memory.set_var_register(x, vy)?;
    }
    memory.set_var_register(0xF, 0b00000001u8 & vx)?;
    memory.set_var_register(x, vx >> 1)
}

fn shift_left(
    memory: &mut Memory,
    x: u8,
    vx: u8,
    vy: u8,
    old_instructions: bool,
) -> Result<(), String> {
    if old_instructions {
        memory.set_var_register(x, vy)?;
    }
    memory.set_var_register(0xF, 0b10000000u8 & vx)?;
    memory.set_var_register(x, vx << 1)
}

fn sub_x_y(memory: &mut Memory, x: u8, vx: u8, vy: u8) -> Result<(), String> {
    let res = vx as i32 - vy as i32;
    if res > 0 {
        memory.set_var_register(0xF, 1)?;
        memory.set_var_register(x, res as u8)
    } else {
        memory.set_var_register(0xF, 0)?;
        memory.set_var_register(x, res as u8)
    }
}

fn sub_y_x(memory: &mut Memory, x: u8, vx: u8, vy: u8) -> Result<(), String> {
    let res = vy as i32 - vx as i32;
    if res > 0 {
        memory.set_var_register(0xF, 1)?;
        memory.set_var_register(x, res as u8)
    } else {
        memory.set_var_register(0xF, 0)?;
        memory.set_var_register(x, (res + 255) as u8)
    }
}

fn add(memory: &mut Memory, x: u8, vx: u8, vy: u8) -> Result<(), String> {
    let res = vx as usize + vy as usize;
    if res > 255 {
        memory.set_var_register(0xF, 1)?;
    } else {
        memory.set_var_register(0xF, 0)?;
    }
    memory.set_var_register(x, res as u8)
}

fn skip_if_neq(x: u8, y: u8, memory: &mut Memory) -> Result<(), String> {
    let vx = memory.get_var_register(x)?;
    let vy = memory.get_var_register(y)?;
    if vx != vy {
        memory.increment_pc();
    }
    Ok(())
}

fn skip_if_eq(x: u8, y: u8, memory: &mut Memory) -> Result<(), String> {
    let vx = memory.get_var_register(x)?;
    let vy = memory.get_var_register(y)?;
    if vx == vy {
        memory.increment_pc();
    }
    Ok(())
}

fn skip_if_neq_im(x: u8, nn: u8, memory: &mut Memory) -> Result<(), String> {
    let vx = memory.get_var_register(x)?;
    if vx != nn {
        memory.increment_pc();
    }
    Ok(())
}

fn skip_if_eq_im(x: u8, nn: u8, memory: &mut Memory) -> Result<(), String> {
    let vx = memory.get_var_register(x)?;
    if vx == nn {
        memory.increment_pc();
    }
    Ok(())
}

fn load_rom(memory: &mut Memory, path: &str) -> Result<(), String> {
    let rom = match fs::read(path) {
        Ok(rom) => rom,
        Err(err) => return Err(format!("Couldn't read rom file: \n {}", err)),
    };
    let pc = memory.pc();
    memory.write_ram(pc, &rom);
    Ok(())
}

fn add_var_register(memory: &mut Memory, x: u8, nn: u8) -> Result<(), String> {
    let curr_var = memory.get_var_register(x)?;
    let result = (nn as usize) + (curr_var as usize);
    memory.set_var_register(x, result as u8)?;
    Ok(())
}

fn zero_instructions(op_code: u16, screen: &mut Screen, memory: &mut Memory) -> Result<(), String> {
    match op_code {
        0x00E0u16 => screen.clear(), // 00E0: clear screen
        0x00EEu16 => {
            // 00EE: return from subroutine
            let adress = memory.pop_stack()?;
            memory.jump_pc(adress)
        }
        _ => return Err("Invalid op code!".to_string()),
    }
    Ok(())
}

fn draw_sprite(
    x: u8,
    y: u8,
    n: u8,
    memory: &mut Memory,
    screen: &mut Screen,
) -> Result<(), String> {
    let index_register = memory.index_register();
    let x_off = memory.get_var_register(x)? % 64;
    let y_off = memory.get_var_register(y)? % 32;
    let mut x_cord;
    let mut y_cord;
    let mut row_sprite_bits;
    let mut vf = 0u8;
    let mut new_pixel;
    let mut curr_pixel;

    for row in 0..n {
        row_sprite_bits = memory.read_ram_cell(index_register + row as u16);
        y_cord = y_off + row;

        for col in 0..8u8 {
            x_cord = x_off + col;
            new_pixel = (row_sprite_bits & (128u8 >> col)) != 0;
            if new_pixel {
                curr_pixel = screen.get_pixel(x_cord, y_cord)?;
                if curr_pixel {
                    vf = 1
                }
                screen.set_pixel(x_cord, y_cord, curr_pixel ^ new_pixel);
            }
            if x_cord >= 63 {
                break;
            }
        }
        if y_cord >= 31 {
            break;
        }
    }
    memory.set_var_register(0xF, vf)?;
    screen.draw();
    Ok(())
}

fn call_subroutine(nnn: u16, memory: &mut Memory) {
    let pc = memory.pc();
    memory.push_stack(pc);
    memory.jump_pc(nnn)
}
