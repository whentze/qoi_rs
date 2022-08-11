#![forbid(unsafe_code)]
#![no_std]

extern crate alloc;
use alloc::{boxed::Box, vec, vec::Vec};
use core::convert::TryInto;

const QOI_INDEX: u8 = 0b00000000;
const QOI_RUN_8: u8 = 0b01000000;
const QOI_RUN_16: u8 = 0b01100000;
const QOI_DIFF_8: u8 = 0b10000000;
const QOI_DIFF_16: u8 = 0b11000000;
const QOI_DIFF_24: u8 = 0b11100000;
const QOI_COLOR: u8 = 0b11110000;

const QOI_MASK_2: u8 = 0b11000000;
const QOI_MASK_3: u8 = 0b11100000;
const QOI_MASK_4: u8 = 0b11110000;

const PADDING: usize = 4;
const HEADER_SIZE: usize = 12;
const MAGIC: [u8; 4] = *b"qoif";

#[derive(Copy, Clone, Default, PartialEq, Eq)]
struct Rgba {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Rgba {
    fn hash(&self) -> u8 {
        self.r ^ self.g ^ self.b ^ self.a
    }
}

pub struct Image {
    pub pixels: Box<[u8]>,
    pub width: u16,
    pub height: u16,
}

pub fn encode(img: Image, channels: u32) -> Option<Box<[u8]>> {
    if img.width == 0 || img.height == 0 || channels < 3 || channels > 4 {
        return None;
    }

    let max_size =
        img.width as usize * img.height as usize * (channels as usize + 1) + HEADER_SIZE + PADDING;
    let mut bytes = Vec::with_capacity(max_size);

    bytes.extend_from_slice(&MAGIC);
    bytes.extend_from_slice(&img.width.to_be_bytes());
    bytes.extend_from_slice(&img.height.to_be_bytes());
    bytes.extend_from_slice(&[0; 4]); // size, will be set later

    let mut index = [Rgba::default(); 64];

    let mut run = 0usize;

    let mut px_prev = Rgba {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    let mut px = px_prev;

    let px_len = img.width as usize * img.height as usize * channels as usize;

    let px_end = px_len - channels as usize;

    for px_pos in (0..px_len).step_by(channels as usize) {
        px.r = *img.pixels.get(px_pos)?;
        px.g = *img.pixels.get(px_pos + 1)?;
        px.b = *img.pixels.get(px_pos + 2)?;
        if channels == 4 {
            px.a = *img.pixels.get(px_pos + 3)?;
        }

        if px == px_prev {
            run += 1;
        }

        if run > 0 && (run == 0x2020 || px != px_prev || px_pos == px_end) {
            if run < 33 {
                run -= 1;
                bytes.push(QOI_RUN_8 | run as u8);
            } else {
                run -= 33;
                bytes.push(QOI_RUN_16 | (run >> 8) as u8);
                bytes.push(run as u8);
            }
            run = 0;
        }

        if px != px_prev {
            let index_pos = px.hash() % 64;
            if *index.get(index_pos as usize)? == px {
                bytes.push(QOI_INDEX | index_pos);
            } else {
                *index.get_mut(index_pos as usize)? = px;

                let vr = px.r as i32 - px_prev.r as i32;
                let vg = px.g as i32 - px_prev.g as i32;
                let vb = px.b as i32 - px_prev.b as i32;
                let va = px.a as i32 - px_prev.a as i32;

                if vr > -16
                    && vr < 17
                    && vg > -16
                    && vg < 17
                    && vb > -16
                    && vb < 17
                    && va > -16
                    && va < 17
                {
                    if va == 0 && vr > -2 && vr < 3 && vg > -2 && vg < 3 && vb > -2 && vb < 3 {
                        bytes.push(
                            QOI_DIFF_8
                                | (((vr + 1) as u8) << 4)
                                | ((vg + 1) as u8) << 2
                                | (vb + 1) as u8,
                        );
                    } else if va == 0
                        && vr > -16
                        && vr < 17
                        && vg > -8
                        && vg < 9
                        && vb > -8
                        && vb < 9
                    {
                        bytes.push(QOI_DIFF_16 | (vr + 15) as u8);
                        bytes.push((((vg + 7) as u8) << 4) | (vb + 7) as u8);
                    } else {
                        bytes.push(QOI_DIFF_24 | (((vr + 15) as u8) >> 1));
                        bytes.push(
                            (((vr + 15) as u8) << 7)
                                | (((vg + 15) as u8) << 2)
                                | (((vb + 15) as u8) >> 3),
                        );
                        bytes.push((((vb + 15) as u8) << 5) | ((va + 15) as u8));
                    }
                } else {
                    bytes.push(
                        QOI_COLOR
                            | if vr != 0 { 8 } else { 0 }
                            | if vg != 0 { 4 } else { 0 }
                            | if vb != 0 { 2 } else { 0 }
                            | if va != 0 { 1 } else { 0 },
                    );
                    if vr != 0 {
                        bytes.push(px.r)
                    }
                    if vg != 0 {
                        bytes.push(px.g)
                    }
                    if vb != 0 {
                        bytes.push(px.b)
                    }
                    if va != 0 {
                        bytes.push(px.a)
                    }
                }
            }
        }
        px_prev = px;
    }
    bytes.extend_from_slice(&[0; PADDING]);

    let data_len = bytes.len() - HEADER_SIZE;

    bytes
        .get_mut(8..12)?
        .copy_from_slice(&(data_len as u32).to_be_bytes());

    Some(bytes.into_boxed_slice())
}

pub fn decode(data: &[u8], channels: u32) -> Option<Image> {
    let header: [u8; HEADER_SIZE] = data.get(0..HEADER_SIZE)?.try_into().ok()?;

    let magic = [header[0], header[1], header[2], header[3]];
    let width = u16::from_be_bytes([header[4], header[5]]);
    let height = u16::from_be_bytes([data[6], data[7]]);
    let size = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

    if channels < 3
        || channels > 4
        || width == 0
        || height == 0
        || size as usize + HEADER_SIZE != data.len()
        || magic != MAGIC
    {
        return None;
    }

    let px_len = width as usize * height as usize * channels as usize;

    let mut pixels = vec![0; px_len].into_boxed_slice();

    let mut px = Rgba {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    let mut index = [Rgba::default(); 64];

    let mut bytes = data[HEADER_SIZE..].iter().cloned();

    let mut run = 0u16;

    for px_pos in (0..px_len).step_by(channels as usize) {
        if run > 0 {
            run -= 1;
        } else {
            let b1 = bytes.next()?;
            if (b1 & QOI_MASK_2) == QOI_INDEX {
                px = *index.get((b1 ^ QOI_INDEX) as usize)?;
            } else if (b1 & QOI_MASK_3) == QOI_RUN_8 {
                run = (b1 & 0x1f) as u16;
            } else if (b1 & QOI_MASK_3) == QOI_RUN_16 {
                let b2 = bytes.next()?;
                run = ((((b1 & 0x1f) as u16) << 8) | (b2 as u16)) + 32;
            } else if (b1 & QOI_MASK_2) == QOI_DIFF_8 {
                px.r = px.r.wrapping_add(((b1 >> 4) & 0x03).wrapping_sub(1));
                px.g = px.g.wrapping_add(((b1 >> 2) & 0x03).wrapping_sub(1));
                px.b = px.b.wrapping_add((b1 & 0x03).wrapping_sub(1));
            } else if (b1 & QOI_MASK_3) == QOI_DIFF_16 {
                let b2 = bytes.next()?;
                px.r = px.r.wrapping_add((b1 & 0x1f).wrapping_sub(15));
                px.g = px.g.wrapping_add((b2 >> 4).wrapping_sub(7));
                px.b = px.b.wrapping_add((b2 & 0x0f).wrapping_sub(7));
            } else if (b1 & QOI_MASK_4) == QOI_DIFF_24 {
                let b2 = bytes.next()?;
                let b3 = bytes.next()?;
                px.r =
                    px.r.wrapping_add((((b1 & 0x0f) << 1) | (b2 >> 7)).wrapping_sub(15));
                px.g = px.g.wrapping_add(((b2 & 0x7c) >> 2).wrapping_sub(15));
                px.b =
                    px.b.wrapping_add((((b2 & 0x03) << 3) | ((b3 & 0xe0) >> 5)).wrapping_sub(15));
                px.a = px.a.wrapping_add((b3 & 0x1f).wrapping_sub(15));
            } else if (b1 & QOI_MASK_4) == QOI_COLOR {
                if b1 & 8 != 0 {
                    px.r = bytes.next()?;
                }
                if b1 & 4 != 0 {
                    px.g = bytes.next()?;
                }
                if b1 & 2 != 0 {
                    px.b = bytes.next()?;
                }
                if b1 & 1 != 0 {
                    px.a = bytes.next()?;
                }
            }
        }

        *index.get_mut(px.hash() as usize % 64)? = px;

        if channels == 3 {
            pixels
                .get_mut(px_pos..(px_pos + 3))?
                .copy_from_slice(&[px.r, px.g, px.b]);
        } else {
            pixels
                .get_mut(px_pos..(px_pos + 4))?
                .copy_from_slice(&[px.r, px.g, px.b, px.a]);
        }
    }

    Some(Image {
        pixels,
        width,
        height,
    })
}

#[cfg(test)]
mod tests;
