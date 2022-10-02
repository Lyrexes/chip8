use bitvec::vec::BitVec;
use bitvec::{bits, bitvec};
use sfml::graphics::{
    Color, PrimitiveType, RenderStates, RenderTarget, RenderWindow, Vertex, View,
};
use sfml::system::Vector2f;
use sfml::window::{ContextSettings, Event, Key, Style, VideoMode};

/*
Screen:
    white_pixels:
        VertexArray with size:
            screen_width * screen_height * verticies_per_rectangle
    pixel_state:
        - stores the color of each pixel
        - either black or white (0 or 1)
    window:
        - sfml RenderWindow
    size:
        - (width,height)
    key_flags:
        - stores key state of every key
    quit_flag:
        - window closed event flag
*/
pub struct Screen {
    white_pixels: Vec<Vertex>,
    pixel_state: BitVec,
    window: RenderWindow,
    size: (u32, u32),
    key_flags: u16,
    quit_flag: bool,
}

impl Screen {
    pub fn new(size: (u32, u32), title: &str) -> Self {
        let mut screen = Screen {
            white_pixels: vec![],
            pixel_state: bitvec![0;64*32],
            window: RenderWindow::new(
                VideoMode::new(size.0, size.1, 32),
                title,
                Style::DEFAULT,
                &ContextSettings::default(),
            ),
            key_flags: 0,
            quit_flag: false,
            size,
        };

        let visible_area = View::new(
            Vector2f::new(size.0 as f32 / 2f32, size.1 as f32 / 2f32),
            Vector2f::new(size.0 as f32, size.1 as f32),
        );
        screen.window.set_view(&visible_area);
        screen.update_screen();
        screen
    }

    pub fn _debug_str(&self) -> String {
        let mut debug_str = String::with_capacity(32 * 64);
        for row in 0..32u8 {
            for col in 0..64u8 {
                debug_str.push(if self.get_pixel(col, row).unwrap() {
                    '*'
                } else {
                    ' '
                });
            }
            debug_str.push('\n');
        }
        debug_str
    }

    pub fn update_screen(&mut self) {
        let pixel_width = self.size.0 as f32 / 64f32;
        let pixel_height = self.size.1 as f32 / 32f32;
        let ones = self.pixel_state.count_ones();
        let pixel_length = self.white_pixels.len();

        if ones > pixel_length {
            self.white_pixels.reserve(ones - pixel_length);
        }
        self.white_pixels.clear();
        self.white_pixels.extend(
            self.pixel_state
                .iter_ones()
                .map(|pixel| {
                    let (row, col) = (pixel / 64 as usize, pixel % 64 as usize);
                    let (x_off, y_off) = (col as f32 * pixel_width, row as f32 * pixel_height);
                    [
                        Vertex::new(
                            Vector2f::new(x_off, y_off),
                            Color::WHITE,
                            Vector2f::default(),
                        ),
                        Vertex::new(
                            Vector2f::new(x_off + pixel_width, y_off),
                            Color::WHITE,
                            Vector2f::default(),
                        ),
                        Vertex::new(
                            Vector2f::new(x_off + pixel_width, y_off + pixel_height),
                            Color::WHITE,
                            Vector2f::default(),
                        ),
                        Vertex::new(
                            Vector2f::new(x_off, y_off + pixel_height),
                            Color::WHITE,
                            Vector2f::default(),
                        ),
                    ]
                })
                .flatten(),
        );
    }

    pub fn handle_events(&mut self) {
        for event in self.window.poll_event() {
            match event {
                Event::Closed => self.quit_flag = true,
                Event::KeyPressed {
                    code: key,
                    alt: _,
                    ctrl: _,
                    shift: _,
                    system: _,
                } => self.key_pressed(key),
                Event::KeyReleased {
                    code: key,
                    alt: _,
                    ctrl: _,
                    shift: _,
                    system: _,
                } => self.key_released(key),
                Event::Resized { width, height } => {
                    self.size = (width, height);
                    let visible_area = View::new(
                        Vector2f::new(width as f32 / 2f32, height as f32 / 2f32),
                        Vector2f::new(width as f32, height as f32),
                    );
                    self.window.set_view(&visible_area);
                    self.draw()
                }
                _ => (),
            }
        }
    }

    fn key_released(&mut self, key: Key) {
        match key {
            Key::X => self.key_flags &= !1u16,
            Key::Num1 => self.key_flags &= !(1u16 << 1),
            Key::Num2 => self.key_flags &= !(1u16 << 2),
            Key::Num3 => self.key_flags &= !(1u16 << 3),
            Key::Q => self.key_flags &= !(1u16 << 4),
            Key::W => self.key_flags &= !(1u16 << 5),
            Key::E => self.key_flags &= !(1u16 << 6),
            Key::A => self.key_flags &= !(1u16 << 7),
            Key::S => self.key_flags &= !(1u16 << 8),
            Key::D => self.key_flags &= !(1u16 << 9),
            Key::Y => self.key_flags &= !(1u16 << 10),
            Key::C => self.key_flags &= !(1u16 << 11),
            Key::Num4 => self.key_flags &= !(1u16 << 12),
            Key::R => self.key_flags &= !(1u16 << 13),
            Key::F => self.key_flags &= !(1u16 << 14),
            Key::V => self.key_flags &= !(1u16 << 15),
            _ => (),
        }
    }

    fn key_pressed(&mut self, key: Key) {
        match key {
            Key::X => self.key_flags |= 1u16,
            Key::Num1 => self.key_flags |= 1u16 << 1,
            Key::Num2 => self.key_flags |= 1u16 << 2,
            Key::Num3 => self.key_flags |= 1u16 << 3,
            Key::Q => self.key_flags |= 1u16 << 4,
            Key::W => self.key_flags |= 1u16 << 5,
            Key::E => self.key_flags |= 1u16 << 6,
            Key::A => self.key_flags |= 1u16 << 7,
            Key::S => self.key_flags |= 1u16 << 8,
            Key::D => self.key_flags |= 1u16 << 9,
            Key::Y => self.key_flags |= 1u16 << 10,
            Key::C => self.key_flags |= 1u16 << 11,
            Key::Num4 => self.key_flags |= 1u16 << 12,
            Key::R => self.key_flags |= 1u16 << 13,
            Key::F => self.key_flags |= 1u16 << 14,
            Key::V => self.key_flags |= 1u16 << 15,
            _ => (),
        }
    }

    pub fn any_key_pressed(&self) -> bool {
        self.key_flags > 0
    }

    pub fn get_pressed_key(&self) -> u8 {
        for key in 0..16u8 {
            if self.key_flags & (1u16 << key) > 0 {
                return key;
            }
        }
        panic!("Function: get_pressed_key was called without checking if a key was pressed!")
    }

    pub fn key_state(&self, key: u8) -> Result<bool, String> {
        if key > 0xF {
            return Err(format!("Invalid key, key must be 0x0-0xF, key: {}", key));
        }
        Ok(self.key_flags & (1u16 << key) > 0)
    }

    pub fn closed(&self) -> bool {
        self.quit_flag
    }

    pub fn draw(&mut self) {
        self.window.clear(Color::BLACK);
        self.update_screen();
        self.window.draw_primitives(
            &self.white_pixels,
            PrimitiveType::QUADS,
            &RenderStates::default(),
        );
        self.window.display();
    }

    pub fn clear(&mut self) {
        self.pixel_state &= bits![0; 64*32];
        self.draw()
    }

    pub fn set_pixel(&mut self, x: u8, y: u8, pixel: bool) {
        self.pixel_state.set(pos_to_index(x, y), pixel);
    }

    pub fn get_pixel(&self, x: u8, y: u8) -> Result<bool, String> {
        match self.pixel_state.get(pos_to_index(x, y)) {
            Some(pixel_state) => Ok(*pixel_state.as_ref()),
            None => Err(format!(
                "Accessed invalid pixel postion: x: {}, y: {}",
                x, y
            )),
        }
    }
}

fn pos_to_index(x: u8, y: u8) -> usize {
    (x as usize) + (y as usize) * 64
}
