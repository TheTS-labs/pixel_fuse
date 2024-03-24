use std::{fs::File, io::Write, time::Duration};
use ring::digest;
use chrono::Local;

use gif::Decoder;

use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::fmt::Write as FmtWrite;
use console::style;

use crate::meta::MetaFrame;

pub struct Read<'a> {
    input: &'a str,
    output: &'a str,
    bar: ProgressBar
}

impl Read<'_> {
    pub fn from<'a>(input: &'a str, output: &'a str) -> Read<'a> {
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
        
        Read { input, output, bar }
    }

    pub fn go(&self) {
        let mut hasher = digest::Context::new(&digest::SHA256);

        let mut decoder = self.get_decoder();

        let meta = self.get_meta_frame(&mut decoder);
        self.println(format!("{} (v{})", style("Decoded meta frame").green().bold(), meta.version));

        if meta.version != env!("CARGO_PKG_VERSION") {
            self.println(format!("{}", style("WARNING: This GIF created with different version of PixelFuse").yellow().bold().underlined()));
        }

        let mut file = File::create(self.output).unwrap();
        self.println(format!(
            "Writing to {}...",
            style(self.output).cyan().underlined(),
        ));

        self.bar.set_message("Decoding GIF...");
        self.bar.set_length(meta.frames_count.try_into().unwrap());
        let mut buffer = self.get_next_frame(&mut decoder).unwrap().clone();
        self.bar.inc(1);

        while let Some(frame) = self.get_next_frame(&mut decoder) {
            file.write(&buffer).unwrap();
            hasher.update(&buffer);
            buffer = frame.clone();
            self.bar.inc(1);
        }

        hasher.update(&buffer);
        file.write(&buffer[..buffer.len()-meta.remainder]).unwrap();

        self.println("Checking hash....");

        let written_hash = format!("{:?}", hasher.finish());

        self.println(format!("Expected hash {}, written hash {}", style(&meta.hash).cyan(), style(&written_hash).cyan()));

        if written_hash == meta.hash {
            self.println(format!("{}", style("Hash matched").green().bold()));
            self.bar.finish();
            return;
        }

        self.println(format!("{}", style("Hash mismatched").red().bold()));
        self.bar.finish();
    }

    fn get_next_frame(&self, decoder: &mut Decoder<File>) -> Option<Vec<u8>> {
        let frame = decoder.read_next_frame().unwrap()?;
        Some(frame.buffer.to_vec())
    }

    fn get_meta_frame(&self, decoder: &mut Decoder<File>) -> MetaFrame {
        let frame = self.get_next_frame(decoder).unwrap();
        MetaFrame::from_bytes(&frame)
    }

    fn get_decoder(&self) -> Decoder<File> {
        let input = File::open(self.input).unwrap();
    
        let mut options = gif::DecodeOptions::new();
        options.set_color_output(gif::ColorOutput::Indexed);
        
        options.read_info(input).unwrap()
    }

    fn println<S: AsRef<str> + std::fmt::Display>(&self, msg: S) {
        self.bar.println(format!("[{}] {msg}", style(Local::now().format("%Y-%m-%d %H:%M:%S%.3f")).blue().bold()));
    }
}