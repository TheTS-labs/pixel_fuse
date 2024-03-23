use std::{fs::File, io::Write};
use sha2::{Sha256, Digest};

use gif::Decoder;

use crate::meta::MetaFrame;

pub struct Read<'a> {
    filename: &'a str
}

impl Read<'_> {
    pub fn from(filename: &str) -> Read {
        Read { filename }
    }

    pub fn go(&self) {
        let mut hasher = Sha256::new();

        let mut decoder = self.get_decoder();
        let meta = self.get_meta_frame(&mut decoder);

        let mut file = File::create(format!("{}.decoded", meta.filename)).unwrap();

        let mut buffer = self.get_next_frame(&mut decoder).unwrap().clone();

        while let Some(frame) = self.get_next_frame(&mut decoder) {
            file.write(&buffer).unwrap();
            hasher.update(&buffer);
            buffer = frame.clone();
        }

        hasher.update(&buffer);
        file.write(&buffer[..buffer.len()-meta.remainder]).unwrap();

        let written_hash = format!("{:x}", hasher.finalize());

        if written_hash == meta.hash {
            println!("Hash matched");
        } else {
            print!("Hash mismatched");
        }
    }

    fn get_next_frame(&self, decoder: &mut Decoder<File>) -> Option<Vec<u8>> {
        let frame = decoder.read_next_frame().unwrap()?;
        Some(self.rgba_to_rgb(frame.buffer.to_vec()))
    }

    fn get_meta_frame(&self, decoder: &mut Decoder<File>) -> MetaFrame {
        let frame = self.get_next_frame(decoder).unwrap();
        MetaFrame::from_bytes(&frame)
    }

    fn get_decoder(&self) -> Decoder<File> {
        let input = File::open(self.filename).unwrap();
    
        let mut options = gif::DecodeOptions::new();
        options.set_color_output(gif::ColorOutput::RGBA);
        
        options.read_info(input).unwrap()
    }

    fn rgba_to_rgb(&self, rgba: Vec<u8>) -> Vec<u8> {
        assert_eq!(rgba.len() % 4, 0);
    
        let mut rgb = Vec::with_capacity(rgba.len() / 4 * 3);
        for chunk in rgba.chunks(4) {
            rgb.extend_from_slice(&chunk[0..3]);
        }
        rgb
    }
}