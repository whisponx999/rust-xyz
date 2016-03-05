extern crate image;
extern crate xyz;

static PNG_DATA: &'static [u8] = include_bytes!("boat2.png");
static XYZ_DATA: &'static [u8] = include_bytes!("boat2.xyz");

#[test]
fn smoke() {
    let _ = read_from_memory(XYZ_DATA);
}

#[test]
fn reading() {
    let png_image = match image::load_from_memory(PNG_DATA) {
        Ok(image::DynamicImage::ImageRgb8(b)) => b,
        _ => panic!("could not load PNG image"),
    };
    let xyz_image = read_from_memory(XYZ_DATA);
    assert_eq!(png_image.width(), xyz_image.width as u32);
    assert_eq!(png_image.height(), xyz_image.height as u32);
    assert_eq!(png_image.into_raw(), xyz_image.to_rgb_buffer());
}

#[test]
fn errors() {
    let result = {
        let mut cursor = PNG_DATA;  // This is not a typo
        xyz::read(&mut cursor)
    };
    assert!(result.is_err());
}

#[test]
fn writing() {
    let original = read_from_memory(XYZ_DATA);
    let reconstituted = {
        let mut out = vec![];
        xyz::write(&original, &mut out).unwrap();
        read_from_memory(&out)
    };
    assert_eq!(original.width, reconstituted.width);
    assert_eq!(original.height, reconstituted.height);
    for (x, y) in original.palette.iter().zip(reconstituted.palette.iter()) {
        assert_eq!(x as &[u8], y as &[u8]);
    }
    assert_eq!(original.buffer, reconstituted.buffer);
}

fn read_from_memory(buf: &[u8]) -> xyz::Image {
    let mut cursor = buf;
    xyz::read(&mut cursor).unwrap()
}
