extern crate std;
use std::{println, vec::Vec};

use glob::glob;

#[test]
fn decode_all() {
    use image::{ImageBuffer, Rgba};

    for entry in glob("reference_images/*/*.png").unwrap() {
        let png_path = entry.unwrap();
        let png_pixels = image::open(&png_path).unwrap().into_rgba8();

        println!("{}", png_path.display());

        let qoi_path = png_path.with_extension("qoi");
        let qoi_bytes = std::fs::read(qoi_path).unwrap();
        let qoi = crate::decode(&qoi_bytes, 4).unwrap();
        let qoi_pixels = ImageBuffer::<Rgba<u8>, _>::from_raw(
            qoi.width as u32,
            qoi.height as u32,
            Vec::from(qoi.pixels),
        )
        .unwrap();

        assert_eq!(png_pixels, qoi_pixels);
    }
}

#[test]
fn encode_all() {
    for entry in glob("reference_images/*/*.png").unwrap() {
        let png_path = entry.unwrap();
        let png_pixels = image::open(&png_path).unwrap().into_rgba8();
        let png_img = crate::Image {
            width: png_pixels.width() as u16,
            height: png_pixels.height() as u16,
            pixels: png_pixels.into_raw().into_boxed_slice(),
        };
        println!("{}", png_path.display());

        let our_qoi = crate::encode(png_img, 4).unwrap();

        let qoi_path = png_path.with_extension("qoi");
        let reference_qoi = std::fs::read(qoi_path).unwrap().into_boxed_slice();

        assert_eq!(our_qoi, reference_qoi);
    }
}

#[test]
fn roundtrip_all() {
    for entry in glob("reference_images/*/*.qoi").unwrap() {
        let reference_qoi = std::fs::read(entry.unwrap()).unwrap().into_boxed_slice();

        let img = crate::decode(&reference_qoi, 4).unwrap();

        let our_qoi = crate::encode(img, 4).unwrap();

        assert_eq!(our_qoi, reference_qoi);
    }
}
