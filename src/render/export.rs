//! Video export by piping raw frames into an external `ffmpeg`.

use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

/// Output container, normally inferred from the file extension.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VideoFormat {
    /// H.264, yuv420p: plays everywhere.
    Mp4,
    /// VP9.
    Webm,
    /// Looping GIF with a generated palette.
    Gif,
}

impl VideoFormat {
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
            "mp4" => Some(Self::Mp4),
            "webm" => Some(Self::Webm),
            "gif" => Some(Self::Gif),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct VideoOptions {
    pub fps: u32,
    /// Quality for mp4/webm: lower is better (x264 default 23; 18 is visually lossless-ish).
    pub crf: u32,
    /// Override the format; `None` infers it from the output extension.
    pub format: Option<VideoFormat>,
    /// ffmpeg executable name or path.
    pub ffmpeg: String,
}

impl Default for VideoOptions {
    fn default() -> Self {
        Self {
            fps: 30,
            crf: 18,
            format: None,
            ffmpeg: "ffmpeg".into(),
        }
    }
}

impl VideoOptions {
    pub fn fps(fps: u32) -> Self {
        Self {
            fps,
            ..Self::default()
        }
    }
}

/// A running ffmpeg process that accepts raw RGBA frames.
pub struct VideoEncoder {
    child: std::process::Child,
    stdin: Option<std::process::ChildStdin>,
}

impl VideoEncoder {
    pub fn start(path: &Path, width: u32, height: u32, opts: &VideoOptions) -> io::Result<Self> {
        let format = opts
            .format
            .or_else(|| VideoFormat::from_path(path))
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("can't infer video format from {path:?}; use .mp4, .webm or .gif"),
                )
            })?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }

        let mut cmd = Command::new(&opts.ffmpeg);
        cmd.args(["-hide_banner", "-loglevel", "error", "-y"])
            .args(["-f", "rawvideo", "-pix_fmt", "rgba"])
            .args(["-s", &format!("{width}x{height}")])
            .args(["-framerate", &opts.fps.to_string()])
            .args(["-i", "-"]);
        match format {
            VideoFormat::Mp4 => {
                cmd.args(["-c:v", "libx264", "-pix_fmt", "yuv420p"])
                    .args(["-crf", &opts.crf.to_string()])
                    // yuv420p needs even dimensions.
                    .args(["-vf", "pad=ceil(iw/2)*2:ceil(ih/2)*2"])
                    .args(["-movflags", "+faststart"]);
            }
            VideoFormat::Webm => {
                cmd.args(["-c:v", "libvpx-vp9", "-pix_fmt", "yuv420p", "-b:v", "0"])
                    .args(["-crf", &opts.crf.to_string()])
                    .args(["-vf", "pad=ceil(iw/2)*2:ceil(ih/2)*2"]);
            }
            VideoFormat::Gif => {
                cmd.args(["-vf", "split[a][b];[a]palettegen[p];[b][p]paletteuse"])
                    .args(["-loop", "0"]);
            }
        }
        cmd.arg(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            if e.kind() == io::ErrorKind::NotFound {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!(
                        "`{}` not found: install ffmpeg (e.g. `brew install ffmpeg`) or set VideoOptions::ffmpeg",
                        opts.ffmpeg
                    ),
                )
            } else {
                e
            }
        })?;
        let stdin = child.stdin.take();
        Ok(Self { child, stdin })
    }

    /// Send one frame of tightly packed RGBA bytes (`width * height * 4`).
    pub fn write_frame(&mut self, rgba: &[u8]) -> io::Result<()> {
        let stdin = self.stdin.as_mut().expect("encoder already finished");
        match stdin.write_all(rgba) {
            Ok(()) => Ok(()),
            // ffmpeg exited early; its stderr has the real reason.
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => Err(self.fail()),
            Err(e) => Err(e),
        }
    }

    /// Close the stream and wait for ffmpeg to finish writing the file.
    pub fn finish(mut self) -> io::Result<()> {
        drop(self.stdin.take());
        let status = self.child.wait()?;
        if status.success() {
            Ok(())
        } else {
            Err(self.stderr_error(status))
        }
    }

    fn fail(&mut self) -> io::Error {
        drop(self.stdin.take());
        match self.child.wait() {
            Ok(status) => self.stderr_error(status),
            Err(e) => e,
        }
    }

    fn stderr_error(&mut self, status: std::process::ExitStatus) -> io::Error {
        let mut msg = String::new();
        if let Some(mut err) = self.child.stderr.take() {
            let _ = err.read_to_string(&mut msg);
        }
        io::Error::other(format!("ffmpeg failed ({status}): {}", msg.trim()))
    }
}

impl Drop for VideoEncoder {
    fn drop(&mut self) {
        // Not finished (error path): don't leave a zombie ffmpeg behind.
        if self.stdin.take().is_some() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_format_from_extension() {
        assert_eq!(
            VideoFormat::from_path(Path::new("a.MP4")),
            Some(VideoFormat::Mp4)
        );
        assert_eq!(
            VideoFormat::from_path(Path::new("a.gif")),
            Some(VideoFormat::Gif)
        );
        assert_eq!(
            VideoFormat::from_path(Path::new("a.webm")),
            Some(VideoFormat::Webm)
        );
        assert_eq!(VideoFormat::from_path(Path::new("a.mov")), None);
        assert_eq!(VideoFormat::from_path(Path::new("noext")), None);
    }

    #[test]
    fn unknown_extension_is_an_error() {
        let e = VideoEncoder::start(Path::new("x.mov"), 4, 4, &VideoOptions::default())
            .err()
            .unwrap();
        assert_eq!(e.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn missing_ffmpeg_gives_a_helpful_error() {
        let opts = VideoOptions {
            ffmpeg: "definitely-not-ffmpeg-xyz".into(),
            ..VideoOptions::default()
        };
        let e = VideoEncoder::start(Path::new("x.mp4"), 4, 4, &opts)
            .err()
            .unwrap();
        assert_eq!(e.kind(), io::ErrorKind::NotFound);
        assert!(e.to_string().contains("install ffmpeg"));
    }
}
