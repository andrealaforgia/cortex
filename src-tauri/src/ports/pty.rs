use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::io::{BufReader, Read, Write};
use std::sync::{Arc, Mutex};

pub struct PtyHandle {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pub child: Arc<Mutex<Box<dyn portable_pty::Child + Send + Sync>>>,
    pair: portable_pty::PtyPair,
}

pub struct PtyReader {
    reader: BufReader<Box<dyn Read + Send>>,
}

impl PtyReader {
    pub fn read_chunk(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

impl PtyHandle {
    pub fn spawn(
        shell: &str,
        cwd: Option<&str>,
        rows: u16,
        cols: u16,
    ) -> Result<(Self, PtyReader), String> {
        let pty_system = native_pty_system();

        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("Failed to open PTY: {}", e))?;

        let mut cmd = CommandBuilder::new(shell);
        if let Some(dir) = cwd {
            cmd.cwd(dir);
        }

        let child = pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("Failed to spawn shell: {}", e))?;

        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("Failed to clone reader: {}", e))?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| format!("Failed to take writer: {}", e))?;

        let handle = PtyHandle {
            writer: Arc::new(Mutex::new(writer)),
            child: Arc::new(Mutex::new(child)),
            pair,
        };

        let pty_reader = PtyReader {
            reader: BufReader::new(reader),
        };

        Ok((handle, pty_reader))
    }

    pub fn write(&self, data: &[u8]) -> Result<(), String> {
        let mut writer = self.writer.lock().map_err(|e| e.to_string())?;
        writer
            .write_all(data)
            .map_err(|e| format!("PTY write error: {}", e))
    }

    pub fn resize(&self, rows: u16, cols: u16) -> Result<(), String> {
        self.pair
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("PTY resize error: {}", e))
    }

    pub fn kill(&self) -> Result<(), String> {
        let mut child = self.child.lock().map_err(|e| e.to_string())?;
        child.kill().map_err(|e| format!("Failed to kill: {}", e))
    }
}
