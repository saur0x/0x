use std::collections::HashMap;

use crate::memory::{Byte, Memory, Word};

use super::instruction_codes;


pub struct CPU {
    pub memory: Memory,
    register_names: Vec<&'static str>,
    registers: Memory,
    pub register_map: HashMap<&'static str, Byte>,
    halt_signal: bool,
}

impl CPU {
    pub fn new(memory: Memory) -> CPU {
        let register_names = vec![
            "r1",
            "r2",
            "r3",
            "r4",
            "r5",
            "r6",
            "r7",
            "r8",
            "pc", // program counter
            "acc", // accumulator
            "sr", // status register
            "sp", // stack pointer
            "fp", // frame pointer
        ];
        let register_names_len = register_names.len();

        let register_map: HashMap<&'static str, Byte> = register_names
            .iter()
            .enumerate()
            .map(|(i, n)| (n.clone(), i as Byte * 4))
            .collect();

        let mut ret = CPU {
            memory,
            register_names,
            registers: Memory::new(register_names_len * 4),
            register_map,
            halt_signal: false,
        };

        // -4 because 4 bytes to store a 32-Bit address
        ret.set_register_by_name("sp", ret.memory.get_size() - 4);
        ret.set_register_by_name("fp", ret.memory.get_size() - 4);

        ret
    }

    fn update_status_register(&mut self, pre: Word, post: Word) {
        let status_register_address = *self.register_map.get("sr").unwrap() as Word;
        if post == 0 {
            self.registers.or_set_byte(status_register_address, 0x01);
        } else {
            self.registers.and_set_byte(status_register_address, 0xFE);
        }

        if pre > post {
            self.registers.or_set_byte(status_register_address, 0x02);
        } else {
            self.registers.and_set_byte(status_register_address, 0xFD);
        }
    }

    fn get_status_flag(&self, flag: Byte) -> bool {
        self.get_register_by_name("sr") & (1u32.wrapping_shl(flag as Word)) != 0
    }

    /// Gets the value of the register with the given name.
    fn get_register_by_name(&self, name: &'static str) -> Word {
        self.registers.get_word(
            *self
                .register_map
                .get(&name)
                .expect(format!("[CPU] get_register: No such register '{}'", name).as_str())
                as Word,
        )
    }

    /// Gets the value of the register with the given address.
    fn get_register_by_address(&self, address: Byte) -> Word {
        self.registers.get_word(address as Word)
    }

    /// Sets the value of the register with the given name.
    fn set_register_by_name(&mut self, name: &'static str, value: Word) {
        self.registers.set_word(
            *self
                .register_map
                .get(&name)
                .expect(format!("[CPU] set_register: No such register '{}'", name).as_str())
                as Word,
            value,
        );
    }

    fn get_program_counter(&self) -> Word {
        self.registers.get_word(*self.register_map.get("pc").unwrap() as Word)
    }

    fn set_program_counter(&mut self, value: Word) {
        self.registers.set_word(*self.register_map.get("pc").unwrap() as Word, value);
    }

    /// Sets the value of the register with the given address.
    fn set_register_by_address(&mut self, address: Byte, value: Word) {
        self.registers.set_word(address as Word, value);
    }

    /// Fetches the next byte from memory and increments the program counter.
    fn fetch_byte(&mut self) -> Byte {
        let next_instruction_address = self.get_program_counter();
        self.set_program_counter(next_instruction_address + 1);

        self.memory.get_byte(next_instruction_address)
    }

    /// Fetches the next word from memory and increments the program counter.
    fn fetch_word(&mut self) -> Word {
        let next_instruction_address = self.get_program_counter();
        self.set_program_counter( next_instruction_address + 4);

        self.memory.get_word(next_instruction_address)
    }

    fn push(&mut self, value: Word) {
        let sp_address = self.get_register_by_name("sp");
        self.memory.set_word(sp_address, value);
        self.set_register_by_name("sp", sp_address - 4);
    }

    fn pop(&mut self) -> Word {
        let next_sp_address = self.get_register_by_name("sp") + 4;
        self.set_register_by_name("sp", next_sp_address);
        return self.memory.get_word(next_sp_address);
    }

    fn execute(&mut self, instruction: Byte) {
        match instruction {
            instruction_codes::HALT => {
                self.halt_signal = true;
            }
            instruction_codes::NOP => {}
            // MOVR 0x0000 1234, r1 -> Move 0x0000 1234 into register r1
            instruction_codes::MOVR => {
                let value = self.fetch_word();
                let register_address = self.fetch_byte();
                self.set_register_by_address(register_address, value);
            }
            // MOVM 0x0000 1234, 0x0000 00AF -> Move 0x0000 1234 into memory at 0x0000 00AF
            instruction_codes::MOVM => {
                let value = self.fetch_word();
                let memory_address = self.fetch_word();
                self.memory.set_word(memory_address, value);
            }
            // MOVRR r1, r2 -> Move register r1 into register r2
            instruction_codes::MOVRR => {
                let register1_address = self.fetch_byte();
                let register2_address = self.fetch_byte();
                self.set_register_by_address(
                    register2_address,
                    self.get_register_by_address(register1_address),
                );
            }
            // MOVRM r1, 0x0000 00AF -> Move register r1 into memory ar 0x0000 00AF
            instruction_codes::MOVRM => {
                let register_address = self.fetch_byte();
                let memory_address = self.fetch_word();
                self.memory.set_word(
                    memory_address,
                    self.get_register_by_address(register_address),
                );
            }
            // MOVMR 0x0000 00AF, r1 -> Move memory at 0x0000 00AF into register r1
            instruction_codes::MOVMR => {
                let memory_address = self.fetch_word();
                let register_address = self.fetch_byte();
                self.set_register_by_address(
                    register_address,
                    self.memory.get_word(memory_address),
                );
            }
            // POP r1 -> Pop value from stack into register r1
            instruction_codes::POP => {
                let register_address = self.fetch_byte();
                let value = self.pop();
                self.set_register_by_address(register_address, value);
            }
            // PUSH 0x0000 1234 -> Push 0x0000 1234 onto stack
            instruction_codes::PUSH => {
                let value = self.fetch_word();
                
                self.push(value);
            }
            // PUSHR r1 -> Push register r1 onto stack
            instruction_codes::PUSHR => {
                let register_address = self.fetch_byte();
                let value = self.get_register_by_address(register_address);

                self.push(value);
            }
            // ADD 0x0000 1234, r1 -> Add 0x0000 1234 to register r1 and store the result in acc
            instruction_codes::ADD => {
                let value = self.fetch_word();
                let register_address = self.fetch_byte();
                let register_value = self.get_register_by_address(register_address);

                let acc = value.wrapping_add(register_value);

                self.set_register_by_name("acc", acc);

                self.update_status_register(value, acc);
            }
            // ADDR r1, r2 -> Add register r1 and register r2 and store the result in acc
            instruction_codes::ADDR => {
                let register1_address = self.fetch_byte();
                let register2_address = self.fetch_byte();

                let register1_value = self.get_register_by_address(register1_address);
                let register2_value = self.get_register_by_address(register2_address);

                let acc = register1_value.wrapping_add(register2_value);

                self.set_register_by_name("acc", acc);

                self.update_status_register(register1_value, acc);
            }
            // BRBS FLAG_Z, 0x0000 00AF -> If the flag Z is set, jump to 0x0000 00AF
            instruction_codes::BRBS => {
                let flag = self.fetch_byte();
                let address = self.fetch_word();
                if self.get_status_flag(flag) {
                    self.set_program_counter( address);
                }
            }
            // BRBC FLAG_Z, 0x0000 00AF -> If the flag Z is clear, jump to 0x0000 00AF
            instruction_codes::BRBC => {
                let flag = self.fetch_byte();
                let address = self.fetch_word();
                if !self.get_status_flag(flag) {
                    self.set_program_counter( address);
                }
            }
            // BREQ 0x0000 1234, 0x0000 0005 -> Jump to 0x0000 0005 if add does equal 0x0000 1234
            instruction_codes::BREQ => {
                let value = self.fetch_word();
                let address = self.fetch_word();

                if self.get_register_by_name("acc") == value {
                    self.set_program_counter( address);
                }
            }
            // BRNQ 0x0000 1234, 0x0000 0005 -> Jump to 0x0000 0005 if acc does not equal 0x0000 1234
            instruction_codes::BRNQ => {
                let value = self.fetch_word();
                let address = self.fetch_word();

                if self.get_register_by_name("acc") != value {
                    self.set_program_counter( address);
                }
            }
            _ => {
                panic!("[CPU] No such instruction: '0x{:02X}'", instruction);
            }
        }
    }

    pub fn debug(&self) {
        for name in self.register_names.clone() {
            println!(
                "{:<4}: 0x{:08X}",
                name,
                self.get_register_by_name(name)
            );
        }
        println!();
    }

    pub fn view_memory_at(&self, address: Word, n: Word) {
        let mut mem_snapshot: Vec<Byte> = Vec::new();
        let max_address = if address + n < self.memory.get_size() {
            address + n
        } else {
            self.memory.get_size()
        };

        for i in address..max_address {
            mem_snapshot.push(self.memory.get_byte(i));
        }

        for (offset, byte) in mem_snapshot.iter().enumerate() {
            if offset % 16 == 0 {
                print!("\n0x{:08X}:", address as usize + offset);
            }
            print!(" 0x{:02X}", byte);
        }
        println!();
    }

    /// Progresses the program
    pub fn step(&mut self) {
        let instruction = self.fetch_byte();
        self.execute(instruction);
    }
}