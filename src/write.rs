use std::{fs::File, io::Read, time::Duration};
use gif::{Frame, Encoder, Repeat};
use ring::digest;
use std::fmt::Write as FmtWrite;
use console::style;
use chrono::Local;
use indicatif::{HumanCount, HumanBytes, HumanFloatCount};

use indicatif::{ProgressBar, ProgressState, ProgressStyle};

use crate::meta::MetaFrame;

pub struct Write<'a> {
  filename: &'a str,
  width: u16,
  height: u16,
  frame_size: usize,
  remainder: Option<usize>,
  hash: Option<String>,
  frames_count: Option<usize>,
  bar: ProgressBar
}

impl Write<'_> {
    pub fn from(filename: &str, width: u16, height: u16) -> Write {
        let bar = ProgressBar::new(0);
        bar.enable_steady_tick(Duration::from_millis(300));
        bar.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed}] {msg}  [{wide_bar:.cyan/blue}] {percent}% |{human_pos}/{human_len}| {per_sec} ({eta})"
            )
            .unwrap()
            .with_key("eta", |state: &ProgressState, w: &mut dyn FmtWrite| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
            .with_key("per_sec", |state: &ProgressState, w: &mut dyn FmtWrite| write!(w, "{} fps", state.per_sec().floor()).unwrap())
            .progress_chars("=> ")
            .tick_strings(&[
                "▹▹▹▹▹",
                "▸▹▹▹▹",
                "▹▸▹▹▹",
                "▹▹▸▹▹",
                "▹▹▹▸▹",
                "▹▹▹▹▸",
                "▪▪▪▪▪",
            ])
        );

        bar.set_message("Preparing...");

        Write {
            filename,
            width,
            height,
            frame_size: (width*height).into(),
            remainder: None,
            hash: None,
            frames_count: None,
            bar
        }
    }

    pub fn go(&mut self) {
        self.bar.set_message("Generating frames...");

        let mut file: Vec<u8> = self.get_file_as_bytes();
        let file_size = file.len();
        self.println(format!(
            "Read {} as {} bytes ({})",
            style(self.filename).cyan().underlined(),
            style(HumanCount(file_size.try_into().unwrap())).cyan(),
            HumanBytes(file_size.try_into().unwrap())
        ));

        let needed_frames = file_size as f32 / self.frame_size as f32;
        let padded_frames = needed_frames.ceil() as usize;

        self.println(format!(
            "Need {} frames to fit the content of the file. Rounding up to {}",
            style(HumanFloatCount(needed_frames.into())).cyan(),
            style(HumanCount(padded_frames.try_into().unwrap())).cyan(),
        ));

        self.remainder = Some((self.frame_size*padded_frames) - file.len());
        self.println(format!("{} pixels will be padded", style(HumanCount(self.remainder.unwrap().try_into().unwrap())).cyan()));

        file.resize(self.frame_size*padded_frames, 0);
        self.println(format!("{}", style("Resized").green().bold()));

        let mut hasher = digest::Context::new(&digest::SHA256);
        hasher.update(&file);
        self.hash = Some(format!("{:?}", hasher.finish()));
        self.println(format!("Calculated frames hash: {}", style(self.hash.clone().unwrap()).green().bold()));

        let frames: Vec<&[u8]> = file.chunks(self.frame_size).collect();
        self.println(format!("{}", style("Chunked file into frames").green().bold()));

        self.frames_count = Some(frames.len());
        self.bar.set_length((frames.len()+1).try_into().unwrap());
        self.bar.set_message("Writing frames...");
        self.write_frames(frames);
    }

    fn generate_meta_frame(&self) -> Frame<'_> {
        self.println("Generating meta frame...");

        let meta_frame = MetaFrame {
            filename: self.filename.to_string(),
            remainder: self.remainder.unwrap(),
            hash: self.hash.clone().unwrap(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            frames_count: self.frames_count.unwrap()
        };
        self.println(format!("{} (v{})", style("Created meta frame").green().bold(), env!("CARGO_PKG_VERSION")));

        let mut bytes = meta_frame.into_bytes();

        assert!(
            bytes.len() <= self.frame_size,
            "Can't fit meta frame in {}x{} frame. Needed space: {}, available space: {}",
            self.width, self.height, bytes.len(), self.frame_size
        );

        self.println(format!("Bytes to write meta frame: {}", HumanCount(bytes.len().try_into().unwrap())));
        
        bytes.resize(self.frame_size, 0);
        
        Frame::from_indexed_pixels(self.width, self.height, &*bytes, None)
    }

    fn write_frames(&self, frames: Vec<&[u8]>) {
        let mut image_file = File::create(format!("{}.gif", self.filename.replace("/", "_"))).unwrap();
        let mut encoder = Encoder::new(&mut image_file, self.width, self.height, &Vec::from_iter(0..255)).unwrap();
        encoder.set_repeat(Repeat::Infinite).unwrap();

        encoder.write_frame(&self.generate_meta_frame()).unwrap();
        self.bar.inc(1);

        for frame in frames {
            let write_frame = Frame::from_indexed_pixels(self.width, self.height, frame, None);
            
            encoder.write_frame(&write_frame).unwrap();

            self.bar.inc(1);
        }

        self.bar.finish();
    }

    fn get_file_as_bytes(&self) -> Vec<u8> {
        let mut file = File::open(self.filename).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();

        buffer
    }

    fn println<S: AsRef<str> + std::fmt::Display>(&self, msg: S) {
        self.bar.println(format!("[{}] {msg}", style(Local::now().format("%Y-%m-%d %H:%M:%S%.3f")).blue().bold()));
    }
}