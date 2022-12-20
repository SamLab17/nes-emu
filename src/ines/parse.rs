use std::error::Error;

use nom::{
    bytes::complete::{tag, take},
    error::context,
    number::complete::le_u8,
    sequence::tuple,
};

use bit::BitIndex;
use derive_try_from_primitive::TryFromPrimitive;

#[repr(u8)]
#[derive(Clone, Copy, Debug, TryFromPrimitive)]
pub enum MirrorType {
    Horizontal = 0,
    Vertical = 1,
}

#[derive(Debug)]
pub struct INesHeader {
    pub prg_rom_size: u32,
    pub chr_rom_size: u32,

    pub prg_ram_size : u32,
    pub prg_nvram_size : u32,

    pub chr_ram_size : u32,
    pub chr_nvram_size : u32,

    // Flags 6
    pub mirror_type: MirrorType,
    pub battery_present: bool,
    pub trainer: bool,
    pub four_screen: bool,

    pub is_ines2: bool,
    pub console_type: u8,
    pub mapper: u16,
    pub submapper: u8,

    pub tv_system: u8,
}

pub struct INesFile {
    pub header: INesHeader,
    pub trainer: Option<Vec<u8>>,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
}

type Input<'a> = &'a [u8];
type ParseResult<'a, O> = nom::IResult<Input<'a>, O, nom::error::VerboseError<Input<'a>>>;

impl INesFile {
    const MAGIC: &'static [u8] = b"NES\x1A";
    const INES2_ID: u8 = 0b10;
    const PRG_ROM_FACTOR: u32 = 16 * 1024;
    const CHR_ROM_FACTOR: u32 = 8 * 1024;
    const RAM_SIZE_SHIFT: u32 = 64;

    fn actual_rom_size(lsb: u8, msb: u8, factor: u32) -> u32 {
        let size = (lsb as u32) | ((msb as u32) << 8);
        if size <= 0xEFF {
            size * factor
        } else {
            let e = size.bit_range(2..8) as u32;
            let m = (size & 0b11) as u32;
            (1 << e) * (m * 2 + 1)
        }
    }

    fn actual_ram_size(shift: u8) -> u32 {
        if shift == 0 {
            0
        } else {
            Self::RAM_SIZE_SHIFT << shift
        }
    }

    fn parse_header(bytes: Input) -> ParseResult<INesHeader> {
        let (bytes, 
             (_, 
              prg_rom_size_lsb,
              chr_rom_size_lsb, 
              flags6, 
              flags7, 
              flags8, 
              flags9,
              flags10,
              flags11,
              flags12,
              _flags13,
              _misc_roms,
              _expansion_dev,
            )) = tuple((
            context("Magic", tag(Self::MAGIC)),
            context("Program ROM Size", le_u8),
            context("Character ROM Size", le_u8),
            context("Flags 6", le_u8),
            context("Flags 7", le_u8),
            context("Flags 8", le_u8),
            context("Flags 9", le_u8),
            context("Flags 10", le_u8),
            context("Flags 11", le_u8),
            context("Flags 12", le_u8),
            context("Flags 13", le_u8),
            context("Miscellaneous ROMs", le_u8),
            context("Default Expansion Device", le_u8),
        ))(bytes)?;

        let ines2 = flags7.bit_range(2..4) != Self::INES2_ID;

        let (prg_rom_size_msb, chr_rom_size_msb) = if ines2 {
            ((flags9 & 0x0f) << 4, flags9 & 0xf0)
        } else {
            (0, 0)
        };

        let mapper = (flags6.bit_range(4..8) as u16)
            | ((flags7 & 0xf0) as u16)
            | ((flags8.bit_range(0..4) as u16) << 8);

        Ok((bytes, 
            INesHeader { 
                prg_rom_size: Self::actual_rom_size(prg_rom_size_lsb, prg_rom_size_msb, Self::PRG_ROM_FACTOR), 
                chr_rom_size: Self::actual_rom_size(chr_rom_size_lsb, chr_rom_size_msb, Self::CHR_ROM_FACTOR),
                prg_ram_size : Self::actual_ram_size(flags10.bit_range(0..4)),
                prg_nvram_size : Self::actual_ram_size(flags10.bit_range(4..8)),
                chr_ram_size : Self::actual_ram_size(flags11.bit_range(0..4)),
                chr_nvram_size : Self::actual_ram_size(flags11.bit_range(4..8)),
                mirror_type : MirrorType::try_from(flags6 & 0b1).unwrap(),
                battery_present : flags6.bit(1),
                trainer : flags6.bit(2),
                four_screen : flags6.bit(3),
                is_ines2 : ines2,
                console_type : flags7.bit_range(0..2),
                mapper : mapper,
                submapper : flags8.bit_range(4..8),
                tv_system : flags12 & 0b11
            })
        )
    }

    fn parse_from(bytes: Input) -> ParseResult<INesFile> {
        let (bytes, header) = context("Header", Self::parse_header)(bytes)?;

        let (bytes, trainer) = if header.trainer {
            let (bytes, trainer) = context("Trainer", take(512u16))(bytes)?;
            (bytes, Some(Vec::from(trainer)))
        } else {
            (bytes, None)
        };

        let (bytes, (prg_rom_ref, chr_rom_ref)) = tuple((
            context("Program ROM", take(header.prg_rom_size)),
            context("Character ROM", take(header.chr_rom_size)),
        ))(bytes)?;

        Ok((
            bytes,
            INesFile {
                header,
                trainer,
                prg_rom: Vec::from(prg_rom_ref),
                chr_rom: Vec::from(chr_rom_ref),
            },
        ))
    }
}

impl TryFrom<&Vec<u8>> for INesFile {
    type Error = Box<dyn Error>;

    fn try_from(file: &Vec<u8>) -> Result<Self, Self::Error> {
        use nom::error::VerboseErrorKind::*;
        use nom::Err::*;
        INesFile::parse_from(file)
            .map(|(_, parsed)| parsed)
            .map_err(|e| {
                match e {
                    // Makes the errors a bit prettier (e.g. hides the byte contents of the file in the error output)
                    Error(ve) => ve
                        .errors
                        .iter()
                        .flat_map(|(_, kind)| match kind {
                            Context(c) => Some(*c),
                            Char(_) => None,
                            Nom(_) => None,
                        })
                        .collect::<Vec<&str>>()
                        .join(" in "),

                    _ => e.to_string(),
                }
                .into()
            })
    }
}
