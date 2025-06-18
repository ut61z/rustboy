fn main() {
    println!("Hello, world!");

    let cpu_freq: u32 = 4_194_304u32; // 4.194304 MHz
    println!("CPU Frequency: {} Hz", cpu_freq);

    let bootrom_start = 0x0000u16; // Boot ROM start address
    let bootrom_end = 0x00FFu16; // Boot ROM end address
    println!("Boot ROM Range: 0x{:04X} - 0x{:04X}", bootrom_start, bootrom_end);
}
