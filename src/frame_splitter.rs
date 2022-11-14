use std::io::Read;
use std::{
    path::{Path, PathBuf},
    process::{ChildStdout, Command, Stdio},
};

use crate::timer::Timer;

#[derive(Debug)]
pub struct FrameSplitter {
    path: PathBuf,
    raw_data_handle: ChildStdout,
    frame_index: usize,
    last_frame: Option<Vec<u8>>,
    pub width: usize,
    pub height: usize,
    fps: (u32, u32),
}

impl FrameSplitter {
    pub fn new(path: &Path) -> Result<Self, std::io::Error> {
        let (width, height) = Self::read_dimensions(path)?;
        let fps = Self::read_fps(path)?;
        let raw_data_handle = Self::initialize_pipe(path)?;
        Ok(Self {
            path: path.to_path_buf(),
            raw_data_handle,
            frame_index: 0,
            last_frame: None,
            width,
            height,
            fps,
        })
    }

    pub fn current_frame(&mut self, timer: &mut Timer) -> Vec<u8> {
        // Check if a new frame needs to be pulled from video
        let needed_frame = timer.elapsed_time().as_millis() as f64;
        let needed_frame = needed_frame / 1000.0;
        let needed_frame = needed_frame * self.fps.0 as f64 / self.fps.1 as f64;
        if self.last_frame == None || needed_frame > self.frame_index as f64 {
            // Pull frame from video
            let frame_size = 4 * self.width * self.height;
            let mut frame = vec![0u8; frame_size];
            self.raw_data_handle.read_exact(&mut frame);
            self.last_frame = Some(frame);
            self.frame_index += 1;
        }

        // Can't be None because of the previous conditional
        self.last_frame.clone().unwrap()
    }

    fn initialize_pipe(path: &Path) -> Result<ChildStdout, std::io::Error> {
        let mut dimensions_cmd = Command::new("ffmpeg");
        let dimensions_cmd = dimensions_cmd
            .arg("-i")
            .arg(path.to_string_lossy().to_string())
            .arg("-v")
            .arg("error")
            .arg("-f")
            .arg("image2pipe")
            .arg("-pix_fmt")
            .arg("rgba")
            .arg("-c:v")
            .arg("rawvideo")
            .arg("-")
            .stdout(Stdio::piped());
        let process = dimensions_cmd.spawn()?;
        let raw_data_handle = process.stdout.unwrap();
        Ok(raw_data_handle)
    }

    fn read_dimensions(path: &Path) -> Result<(usize, usize), std::io::Error> {
        let mut dimensions_cmd = Command::new("ffprobe");
        let dimensions_cmd = dimensions_cmd
            .arg("-i")
            .arg(path.to_string_lossy().to_string())
            .arg("-v")
            .arg("error")
            .arg("-select_streams")
            .arg("v")
            .arg("-show_entries")
            .arg("stream=width,height")
            .arg("-of")
            .arg("csv=p=0:s=x");
        println!("{}", path.display());
        let dimensions_output = dimensions_cmd.output()?.stdout;
        let output = String::from_utf8_lossy(&dimensions_output);
        let mut dimensions = output.trim().split('x');
        let width = dimensions.next().unwrap().parse::<usize>().unwrap();
        let height = dimensions.next().unwrap().parse::<usize>().unwrap();
        Ok((width, height))
    }

    fn read_fps(path: &Path) -> Result<(u32, u32), std::io::Error> {
        let mut dimensions_cmd = Command::new("ffprobe");
        let dimensions_cmd = dimensions_cmd
            .arg("-i")
            .arg(path.to_string_lossy().to_string())
            .arg("-v")
            .arg("error")
            .arg("-select_streams")
            .arg("v")
            .arg("-show_entries")
            .arg("stream=r_frame_rate")
            .arg("-of")
            .arg("default=noprint_wrappers=1:nokey=1");
        println!("{}", path.display());
        let dimensions_output = dimensions_cmd.output()?.stdout;
        let output = String::from_utf8_lossy(&dimensions_output);
        let mut dimensions = output.trim().split('/');
        let num = dimensions.next().unwrap().parse::<u32>().unwrap();
        let denom = dimensions.next().unwrap().parse::<u32>().unwrap();
        Ok((num, denom))
    }
}
