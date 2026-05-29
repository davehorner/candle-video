use anyhow::{bail, Context};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

pub fn save_mp4_from_rgb_frames(
    path: impl AsRef<Path>,
    frames: &[Vec<u8>],
    width: usize,
    height: usize,
    fps: usize,
) -> anyhow::Result<()> {
    if frames.is_empty() {
        bail!("cannot encode mp4 without frames");
    }

    let expected_len = width * height * 3;
    for (idx, frame) in frames.iter().enumerate() {
        if frame.len() != expected_len {
            bail!(
                "frame {idx} has {} bytes, expected {expected_len} for {width}x{height} RGB",
                frame.len()
            );
        }
    }

    let output_path = path.as_ref().to_string_lossy().to_string();
    let size = format!("{width}x{height}");
    let fps = fps.to_string();

    let mut child = Command::new("ffmpeg")
        .args([
            "-y",
            "-loglevel",
            "error",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "-s",
            &size,
            "-r",
            &fps,
            "-i",
            "pipe:0",
            "-an",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
            &output_path,
        ])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| "failed to start ffmpeg; install ffmpeg and ensure it is on PATH")?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .context("failed to open ffmpeg stdin")?;
        for frame in frames {
            stdin
                .write_all(frame)
                .context("failed writing raw frames to ffmpeg")?;
        }
    }

    let output = child
        .wait_with_output()
        .context("failed waiting for ffmpeg")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("ffmpeg failed while writing {output_path}: {stderr}");
    }

    Ok(())
}
