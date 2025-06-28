use image::{GenericImage, GenericImageView, Pixel, SubImage};

use crate::{encryption_key::EncryptionKey, error::Error};
use std::io::{BufReader, Read};

const DEFAULT_PNG_HEADER: [u8; 16] = [
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
];

pub fn decrypt<R>(key: Option<EncryptionKey>, data: R) -> Result<Vec<u8>, Error>
where
    R: Read,
{
    let reader = BufReader::new(data);
    let bytes = reader.bytes().collect::<Result<Vec<_>, _>>()?;

    let key = match key {
        Some(key) => key,
        None => {
            let key = EncryptionKey::new(DEFAULT_PNG_HEADER).xor(&bytes[16..32]);

            EncryptionKey::new(std::array::from_fn(|i| key[i]))
        }
    };

    let decrypted_header = key.xor(&bytes[16..32]);
    let body = &bytes[32..];

    let mut output = Vec::with_capacity(decrypted_header.len() + body.len());

    output.extend(decrypted_header);
    output.extend(body);

    Ok(output)
}

#[inline(always)]
pub fn decrypt_derive_key<R>(data: R) -> Result<Vec<u8>, Error>
where
    R: Read,
{
    decrypt(None, data)
}

#[derive(Debug)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SpritesheetKind {
    Character,
    Face,
}
impl SpritesheetKind {
    pub const fn sprite_count(&self) -> Point<usize> {
        match self {
            Self::Character => Point { x: 12, y: 8 },
            Self::Face => Point { x: 3, y: 4 },
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ImageData(Vec<u8>);

#[ouroboros::self_referencing]
struct SpritesheetImage<Image>
where
    Image: GenericImage + 'static,
{
    image: Image,
    #[borrows(image)]
    #[not_covariant]
    subimages: Vec<SubImage<&'this Image>>,
}

// #[derive(Debug, PartialEq)]
pub struct Spritesheet<Image>
where
    Image: GenericImage + 'static,
{
    pub kind: SpritesheetKind,
    image: SpritesheetImage<Image>,
}
impl<I> Spritesheet<I>
where
    I: GenericImage + 'static,
{
    pub fn new(kind: SpritesheetKind, image: I) -> Self {
        let width = image.width() as usize;
        let height = image.height() as usize;

        let Point { x: col_count, y: row_count } = kind.sprite_count();

        let sprite_width = width / col_count;
        let sprite_height = height / row_count;

        let sprite_count = col_count * row_count;

        let get_pos_from_index = |index: usize| {
            let row_size = width / sprite_width;

            let x_index = index % row_size;
            let y_index = index / row_size;

            Point {
                x: x_index * sprite_width,
                y: y_index * sprite_height,
            }
        };

        Self {
            kind,
            image: SpritesheetImageBuilder {
                image,
                subimages_builder: |image| {
                    (0..sprite_count)
                        .map(|i| {
                            let sprite_pos = get_pos_from_index(i);
                            image.view(
                                sprite_pos.x as u32,
                                sprite_pos.y as u32,
                                sprite_width as u32,
                                sprite_height as u32,
                            )
                        })
                        .collect()
                },
            }
            .build(),
        }
    }

    pub fn get_subimage(&self, index: usize) -> Option<&SubImage<&I>> {
        self.image.with_subimages(|subimages| subimages.get(index))
    }
}
