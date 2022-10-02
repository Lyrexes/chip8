pub struct Memory {
    ram: [u8; 4096],
    stack: Vec<u16>,
    index_register: u16,
    delay_register: u8,
    sound_register: u8,
    program_counter: u16,
    var_registers: [u8; 16],
}

impl Memory {
    pub fn new() -> Self {
        let mut ram = [0; 4096];

        let fonts = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        for i in 0..fonts.len() {
            ram[0x0050 + i] = fonts[i];
        }

        Memory {
            ram,
            stack: vec![],
            index_register: 0,
            delay_register: 0,
            sound_register: 0,
            var_registers: [0; 16],
            program_counter: 0x200, // start adress
        }
    }

    pub fn set_index_register(&mut self, adress: u16) {
        self.index_register = adress
    }

    pub fn index_register(&self) -> u16 {
        self.index_register
    }

    pub fn set_var_register(&mut self, id: u8, var: u8) -> Result<(), String> {
        if id > 15 {
            return Err(format!(
                "Var register id is out of range, must be 0x0-0xF id: {}",
                id
            ));
        }
        self.var_registers[id as usize] = var;
        Ok(())
    }

    pub fn get_var_register(&self, id: u8) -> Result<u8, String> {
        if id > 15 {
            return Err(format!(
                "Var register id is out of range, must be 0x0-0xF id: {}",
                id
            ));
        }
        Ok(self.var_registers[id as usize])
    }

    pub fn jump_pc(&mut self, adress: u16) {
        self.program_counter = adress;
    }

    pub fn increment_pc(&mut self) {
        self.program_counter += 2;
    }

    pub fn pc(&self) -> u16 {
        self.program_counter
    }

    pub fn pop_stack(&mut self) -> Result<u16, String> {
        match self.stack.pop() {
            Some(val) => Ok(val),
            None => return Err("pop called on empty stack".to_string()),
        }
    }

    pub fn push_stack(&mut self, adress: u16) {
        self.stack.push(adress)
    }

    pub fn fetch_instruction(&self) -> (u8, u8) {
        (
            self.ram[self.program_counter as usize],
            self.ram[self.program_counter as usize + 1],
        )
    }

    pub fn write_ram(&mut self, address: u16, mem: &[u8]) {
        for i in 0..mem.len() {
            self.ram[i + address as usize] = mem[i];
        }
    }

    pub fn read_ram_cell(&self, address: u16) -> u8 {
        self.ram[address as usize]
    }

    pub fn decrement_sound(&mut self) {
        if self.sound_register != 0 {
            self.sound_register -= 1;
        }
    }

    pub fn decrement_delay(&mut self) {
        if self.delay_register != 0 {
            self.delay_register -= 1;
        }
    }

    pub fn delay_register(&self) -> u8 {
        self.delay_register
    }

    pub fn set_delay_register(&mut self, register: u8) {
        self.delay_register = register
    }

    pub fn set_sounds_register(&mut self, register: u8) {
        self.sound_register = register
    }

    pub fn decrement_pc(&mut self) {
        self.program_counter -= 2
    }
}
