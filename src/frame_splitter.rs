use std::io::Read;
use std::{
    path::{Path, PathBuf},
    process::{ChildStdout, Command, Stdio},
};

#[derive(Debug)]
pub struct FrameSplitter {
    path: PathBuf,
    raw_data_handle: ChildStdout,
    frame_index: usize,
    pub width: usize,
    pub height: usize,
}

impl FrameSplitter {
    pub fn new(path: &Path) -> Result<Self, std::io::Error> {
        // Command::new("ffmpeg -i my-immortal.webm -f image2pipe -pix_fmt rgb8 -c:v rawvideo -");
        let dimensions = Self::read_dimensions(path)?;
        let (width, height) = dimensions;
        let raw_data_handle = Self::initialize_pipe(path)?;
        Ok(Self {
            path: path.to_path_buf(),
            raw_data_handle,
            frame_index: 0,
            width,
            height,
        })
    }

    pub fn next_frame(&mut self) -> Vec<u8> {
        let frame_size = 4 * self.width * self.height;
        let mut frame = vec![0u8; frame_size];
        self.raw_data_handle.read_exact(&mut frame);
        frame
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
}
