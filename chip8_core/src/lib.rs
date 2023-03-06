use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
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

pub struct Emu {
    pc: u16, // pc = program counter
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16, // Stack pointer
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8, // Delay timer
    st: u8, // Sound timer
}

const START_ADDR: u16 = 0x200; // The adress where the rom is loaded at.

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn get_screen(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self) {
        let op = self.fetch();

        self.execute(op);
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;

        op
    }

    pub fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        println!("Executing code {:X}", op);

        match (digit1, digit2, digit3, digit4) {
            // ✅
            // NOP 0x0000
            (0, 0, 0, 0) => return,

            // ✅
            // CLS 0x00E0
            (0, 0, 0xE, 0) => self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT],

            // ✅
            // RET 0x00EE
            (0, 0, 0xE, 0xE) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            }

            // ✅
            // JMP 0x1NNN
            (1, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = nnn;
            }

            // ✅
            // CALL 0x2NNN
            (2, _, _, _) => {
                let nnn = op & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            }
            // ✅
            // SKIP 0x3XNN X == NN
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }

            // ✅
            // SKIP 0x4XNN X != NN
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            }

            // SKIP 0x5XY0 X == Y
            (5, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }

            // ✅
            // Set v_reg 0x6XNN X = NN
            (6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;

                self.v_reg[x] = nn;
            }

            // ✅
            // ADD 0x7XNN X += NN
            (7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            }

            // ✅
            // SET v_reg 0x8XY0 X = Y
            (8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y];
            }

            // ✅
            // SET v_reg 0x8XY1 X |= Y
            (8, _, _, 1) => {
                let x = digit2 as usize;
                let y = digit2 as usize;

                self.v_reg[x] |= self.v_reg[y];
            }

            // ✅
            // SET v_reg 0x8XY2 X &= Y
            (8, _, _, 2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                self.v_reg[x] &= self.v_reg[y];
            }

            // SET v_reg 0x8XY3 X ^= Y
            (8, _, _, 3) => {
                let x = digit2 as usize;
                let y = digit2 as usize;

                self.v_reg[x] ^= self.v_reg[y];
            }
            // ✅
            // ADD v_regs 0x8XY4 X += Y
            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_x, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);

                let new_f = if carry { 1 } else { 0 };
                self.v_reg[x] = new_x;
                self.v_reg[0xF] = new_f;
            }

            // SUB v_regs 0x8XY5 X -= Y
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_x, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_x;
                self.v_reg[0xF] = new_vf;
            }

            // ✅
            // bitshift 0x8XY6 x >>= 1
            (8, _, _, 6) => {
                let x = digit2 as usize;
                let dropped_bit = self.v_reg[x] & 1;

                self.v_reg[x] >>= 1;
                self.v_reg[0xf] = dropped_bit;
            }

            // SUB v_regs 0x8XY7 X = Y-X
            (8, _, _, 7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_x, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_x;
                self.v_reg[0xF] = new_vf;
            }

            // ✅
            // bitshift 0x8XYE X <<= 1
            (8, _, _, 0xE) => {
                let x = digit2 as usize;
                let dropped_bit = (self.v_reg[x] >> 7) & 1;

                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = dropped_bit;
            }
            // Skip 0x9XY0 X != Y
            (9, _, _, 0) => {
                let x = digit3 as usize;
                let y = digit3 as usize;

                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            }
            // ✅
            // Write to ireg 0xANNN
            (0xA, _, _, _) => {
                let nnn = op & 0x0FFF;
                self.i_reg = nnn;
            }

            // Jump 0xBNNN v_reg[0] + NNN
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            }

            // Random masked 0xCXNN
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            }

            // ✅
            // Draw 0xDXYN
            (0xD, _, _, _) => {
                let x_coord = self.v_reg[digit2 as usize] as u16;
                let y_coord = self.v_reg[digit3 as usize] as u16;
                let num_rows = digit4;

                let mut flipped = false;

                for y_line in 0..num_rows {
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];

                    for x_line in 0..8 {
                        if (pixels & (0b1000_0000) >> x_line) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            let idx = x + SCREEN_WIDTH * y;

                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            }

            // SKIP if key pressed 0xEX9E
            (0xE, _, 9, 0xE) => {
                let idx = self.v_reg[digit2 as usize];
                let key = self.keys[idx as usize];

                if key {
                    self.pc += 2;
                }
            }

            // ✅
            // SKIP if key not pressed 0xEXA1
            (0xE, _, 0xA, 1) => {
                let idx = self.v_reg[digit2 as usize];
                let key = self.keys[idx as usize];

                if !key {
                    self.pc += 2;
                }
            }

            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.v_reg[x] = self.dt;
            }

            // WAIT KEY 0xFX0A
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    self.pc -= 2;
                }
            }

            // Set Delay timer 0xFX15
            (0xF, _, 1, 5) => {
                let x = digit2 as usize;
                self.dt = self.v_reg[x];
            }

            // Set Sound timer 0xFX18
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.st = self.v_reg[x];
            }
            // ✅
            // ADD 0xFX1E i += X
            (0xF, _, 1, 0xE) => {
                let val = self.v_reg[digit2 as usize] as u16;
                self.i_reg = self.i_reg.wrapping_add(val);
            }

            // SET i_reg to font address 0xFX29
            (0xF, _, 2, 9) => {
                let val = self.v_reg[digit2 as usize] as u16;
                self.i_reg = val * 5;
            }

            // VREG to BCD 0xFX33
            (0xF, _, 3, 3) => {
                let val = self.v_reg[digit2 as usize] as f32;
                let hundreds = (val / 100.0).floor() as u8;
                let tens = ((val / 10.0) % 10.0).floor() as u8;
                let ones = (val % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }

            // STORE 0xFX55 V0..X
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.ram[i + idx] = self.v_reg[idx]
                }
            }
            // ✅
            // LOAD 0xFX65 V0..X
            (0xF, _, 6, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;

                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[i + idx]
                }
            }

            (_, _, _, _) => unimplemented!("Unimplemented opcode: {:0x}", op),
        }
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // BEEP
            }
            self.st -= 1;
        }
    }
}