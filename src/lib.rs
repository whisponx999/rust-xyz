extern crate byteorder;
extern crate flate2;

use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use flate2::{FlateReadExt, FlateWriteExt, Compression};
use std::io::{self, Read, Write};

const MAGIC_NUMBER: u32 = 0x315a5958;  // "XYZ1"

/// Represents an XYZ image.
pub struct Image {
    /// Image height in buffer.
    pub width: u16,
    /// Image width in buffer.
    pub height: u16,
    /// List of colors used by the image.
    pub palette: [Color; 256],
    /// Image data. This contains `width * height` bytes, one for each
    /// pixel in the image. The color of each pixel is determined by the
    /// `palette` array.
    pub buffer: Vec<u8>,
}

/// Represents a color in RGB form.
pub type Color = [u8; 3];

impl Image {
    /// Converts the image to a raw RGB buffer, suitable for the Piston
    /// `image` library.
    pub fn to_rgb_buffer(&self) -> Vec<u8> {
        self.buffer.iter()
            .flat_map(|&i| self.palette[i as usize].iter().cloned())
            .collect()
    }
}

// XXX: https://github.com/BurntSushi/byteorder/pull/40
#[allow(non_snake_case)]
fn B(e: byteorder::Error) -> io::Error {
    use byteorder::Error::*;
    match e {
        UnexpectedEOF => io::Error::new(io::ErrorKind::UnexpectedEof, UnexpectedEOF),
        Io(e) => e
    }
}

/// Reads an XYZ image.
pub fn read<R: Read>(reader: &mut R) -> io::Result<Image> {
    let magic = try!(reader.read_u32::<LittleEndian>());
    if magic != MAGIC_NUMBER {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid XYZ header"));
    }

    let width = try!(reader.read_u16::<LittleEndian>());
    let height = try!(reader.read_u16::<LittleEndian>());

    let mut decompress = vec![].zlib_decode();
    try!(io::copy(reader, &mut decompress));
    let body = try!(decompress.finish());
    let mut body = &body as &[u8];

    let mut palette = [[0u8; 3]; 256];
    for slot in palette.iter_mut() {
        try!(body.read_exact(slot));
    }

    let mut buffer = vec![0u8; (width as usize) * (height as usize)];
    try!(body.read_exact(&mut buffer));

    if !body.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "extra data at end of XYZ file"));
    }

    Ok(Image {
        width: width,
        height: height,
        palette: palette,
        buffer: buffer,
    })
}

/// Writes an XYZ image.
pub fn write<W: Write>(image: &Image, writer: &mut W) -> io::Result<()> {
    try!(writer.write_u32::<LittleEndian>(MAGIC_NUMBER).map_err(B));

    try!(writer.write_u16::<LittleEndian>(image.width).map_err(B));
    try!(writer.write_u16::<LittleEndian>(image.height).map_err(B));

    let mut compress = writer.zlib_encode(Compression::Default);
    for slot in image.palette.iter() {
        try!(compress.write_all(slot));
    }
    try!(compress.write_all(&image.buffer));
    try!(compress.finish());

    Ok(())
}
