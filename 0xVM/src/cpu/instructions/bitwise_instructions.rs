use crate::cpu::CPU;

macro_rules! instr_helper {
    ($cpu:ident, $val1:ident, $f:ident, $val2:ident, $destination:ident) => {
        // calculate the result with given function $f, update $destination with result
        // and update status register
        let res = $val1.$f($val2);

        $cpu.set_reg($destination, res);

        $cpu.update_sr($val1, res);
    };

	($cpu:ident, $val1:ident, $op:tt, $val2:ident, $destination:ident) => {
        // calculate the result with given function $f, update $destination with result
        // and update status register
        let res = $val1 $op $val2;

        $cpu.set_reg($destination, res);

        $cpu.update_sr($val1, res);
    };

	(rw, $cpu:ident) => {{
		// fetch register value
        let r_addr = $cpu.fetch_word();
        let r_val = $cpu.get_reg(r_addr);

        // fetch literal value
        let value = $cpu.fetch_word();

		(r_addr, r_val, value)
	}};

	(rr, $cpu:ident) => {{
		// fetch register values
		let r1_addr = $cpu.fetch_word();
        let r2_addr = $cpu.fetch_word();

        let r1_val = $cpu.get_reg(r1_addr);
        let r2_val = $cpu.get_reg(r2_addr);

		(r1_addr, r2_addr, r1_val, r2_val)
	}};
}

macro_rules! instr {
    // use functions
    ($cpu:ident, rw, $f:ident) => {
        let (r_addr, r_val, value) = instr_helper!(rw, $cpu);

        instr_helper!($cpu, r_val, $f, value, r_addr);
    };

    ($cpu:ident, rr, $f:ident) => {
        let (r1_addr, _r2_addr, r1_val, r2_val) = instr_helper!(rr, $cpu);

        instr_helper!($cpu, r1_val, $f, r2_val, r1_addr);
    };

    // use operators
    ($cpu:ident, rw, $op:tt) => {
        let (r_addr, r_val, value) = instr_helper!(rw, $cpu);

        instr_helper!($cpu, r_val, $op, value, r_addr);
    };

    ($cpu:ident, rr, $op:tt) => {
        let (r1_addr, _r2_addr, r1_val, r2_val) = instr_helper!(rr, $cpu);

        instr_helper!($cpu, r1_val, $op, r2_val, r1_addr);
    };
}

/// ## LSF r1, 0x4
/// Shift register r1 left by 0x4
#[inline]
#[allow(non_snake_case)]
pub fn LSF(cpu: &mut CPU) {
    instr!(cpu, rw, wrapping_shl);
}

/// ## LSFR r1, r2
/// Shift register r1 left by register r2
#[inline]
#[allow(non_snake_case)]
pub fn LSFR(cpu: &mut CPU) {
    instr!(cpu, rr, wrapping_shl);
}

/// ## RSF r1, 0x4
/// Shift register r1 right by 0x4
#[inline]
#[allow(non_snake_case)]
pub fn RSF(cpu: &mut CPU) {
    instr!(cpu, rw, wrapping_shr);
}

/// ## RSFR r1, r2
/// Shift register r1 right by register r2
#[inline]
#[allow(non_snake_case)]
pub fn RSFR(cpu: &mut CPU) {
    instr!(cpu, rr, wrapping_shr);
}

/// ## WLSF r1, 0x4
/// Shift register r1 left by 0x4 wrapping around
#[inline]
#[allow(non_snake_case)]
pub fn WLSF(cpu: &mut CPU) {
    instr!(cpu, rw, rotate_left);
}

/// ## WLSFR r1, r2
/// Shift register r1 left by register r2 wrapping around
#[inline]
#[allow(non_snake_case)]
pub fn WLSFR(cpu: &mut CPU) {
    instr!(cpu, rr, rotate_left);
}

/// ## WRSF r1, 0x4
/// Shift register r1 right by 0x4 wrapping around
#[inline]
#[allow(non_snake_case)]
pub fn WRSF(cpu: &mut CPU) {
    instr!(cpu, rw, rotate_right);
}

/// ## WRSFR r1, r2
/// Shift register r1 right by register r2 wrapping around
#[inline]
#[allow(non_snake_case)]
pub fn WRSFR(cpu: &mut CPU) {
    instr!(cpu, rr, rotate_right);
}

/// ## AND r1, 0x4
/// Bitwise AND register r1 with 0x4
#[inline]
#[allow(non_snake_case)]
pub fn AND(cpu: &mut CPU) {
    instr!(cpu, rw, &);
}

/// ## ANDR r1, r2
/// Bitwise AND register r1 with register r2
#[inline]
#[allow(non_snake_case)]
pub fn ANDR(cpu: &mut CPU) {
    instr!(cpu, rr, &);
}

/// ## OR r1, 0x4
/// Bitwise OR register r1 with 0x4
#[inline]
#[allow(non_snake_case)]
pub fn OR(cpu: &mut CPU) {
	instr!(cpu, rw, |);
}

/// ## ORR r1, r2
/// Bitwise OR register r1 with register r2
#[inline]
#[allow(non_snake_case)]
pub fn ORR(cpu: &mut CPU) {
	instr!(cpu, rr, |);
}

/// ## XOR r1, 0x4
/// Bitwise XOR register r1 with 0x4
#[inline]
#[allow(non_snake_case)]
pub fn XOR(cpu: &mut CPU) {
	instr!(cpu, rw, ^);
}

/// ## XORR r1, r2
/// Bitwise XOR register r1 with register r2
#[inline]
#[allow(non_snake_case)]
pub fn XORR(cpu: &mut CPU) {
	instr!(cpu, rr, ^);
}

/// ## NOT r1
/// Bitwise NOT register r1
#[inline]
#[allow(non_snake_case)]
pub fn NOT(cpu: &mut CPU) {
    let r_addr = cpu.fetch_word();
    let register_val = cpu.get_reg(r_addr);
    let res = !register_val;

    cpu.set_reg(r_addr, res);

    cpu.update_sr(register_val, res);
}
