use criterion::{criterion_group, criterion_main, Criterion};
use nes_emu::cpu::cpu::Cpu;
use nes_emu::cart::builder::build_cartridge;
use nes_emu::ines::parse::INesFile;
use std::fs;
use std::time::Duration;

pub fn criterion_benchmark(c: &mut Criterion) {
    const ROM: &'static str = "roms/smb.nes";
    let rom = fs::read(ROM).expect("Could not find ROM at given path.");
    let ines_rom = INesFile::try_from(&rom).expect("Path provided is not a valid NES ROM.");

    let mut cpu = Cpu::new(
        build_cartridge(&ines_rom).expect("This ROM is not supported."),
        None,
        None,
    );
    let mut group = c.benchmark_group("cpu");
    group.measurement_time(Duration::from_secs(10));
    // c.measurement_time(Duration::from_secs(10));
    group.bench_function("next_frame", |b| b.iter(|| cpu.next_frame().unwrap()));
}


criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);