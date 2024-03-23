use std::{fs::File, io::Read};
use gif::{Frame, Encoder, Repeat};
use sha2::{Sha256, Digest};

use crate::meta::MetaFrame;

pub struct Write<'a> {
  filename: &'a str,
  width: u16,
  height: u16,
  frame_size: usize,
  remainder: Option<usize>,
  hash: Option<String>
}

impl Write<'_> {
    pub fn from(filename: &str, width: u16, height: u16) -> Write {
        Write {
            filename,
            width,
            height,
            frame_size: (width*height*3).into(),
            remainder: None,
            hash: None
        }
    }

    pub fn go(&mut self) {
        let mut file: Vec<u8> = self.get_file_as_bytes();
        let file_size = file.len();

        let needed_frames = file_size as f32 / self.frame_size as f32;
        let padded_frames = needed_frames.ceil() as usize;

        self.remainder = Some((self.frame_size*padded_frames) - file.len());
        file.resize(self.frame_size*padded_frames, 0);

        let mut hasher = Sha256::new();
        hasher.update(&file);
        self.hash = Some(format!("{:x}", hasher.finalize()));

        let frames: Vec<&[u8]> = file.chunks(self.frame_size).collect();

        self.write_frames(frames);
    }

    fn generate_meta_frame(&self) -> Frame<'_> {
        let meta_frame = MetaFrame {
            filename: self.filename.to_string(),
            remainder: self.remainder.unwrap(),
            hash: self.hash.clone().unwrap()
        };
        let mut bytes = meta_frame.into_bytes();

        assert!(
            bytes.len() <= self.frame_size,
            "Can't fit meta frame in {}x{} ({}*{}*3) frame. Needed space: {}, available space: {}",
            self.width, self.height, self.width, self.height, bytes.len(), self.frame_size
        );
        
        bytes.resize(self.frame_size, 0);
        
        Frame::from_rgb(self.width, self.height, &bytes)
    }

    fn write_frames(&self, frames: Vec<&[u8]>) {
        let mut image_file = File::create(format!("{}.gif", self.filename.replace("/", "_"))).unwrap();
        let mut encoder = Encoder::new(&mut image_file, self.width, self.height, &[]).unwrap();
        encoder.set_repeat(Repeat::Infinite).unwrap();

        encoder.write_frame(&self.generate_meta_frame()).unwrap();

        for frame in frames {
            let write_frame = Frame::from_rgb(self.width, self.height, frame);
            
            encoder.write_frame(&write_frame).unwrap();
        }
    }

    fn get_file_as_bytes(&self) -> Vec<u8> {
        let mut file = File::open(self.filename).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        buffer
    }
}