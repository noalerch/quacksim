use std::io::{self, Write};

use quacksim::{TickRecorder, TickReport};

use crate::stats::Statistics;

// WSG impl of CsvRecorder
pub struct CsvRecorder<W> {
    writer: W,
}

impl<W: Write> CsvRecorder<W> {
    pub fn new(mut writer: W) -> io::Result<Self> {
        writer.write_all(b"tick,sheep,wolves,grown_grass\n")?;
        Ok(Self { writer })
    }

    pub fn finish(mut self) -> io::Result<W> {
        self.writer.flush()?;
        Ok(self.writer)
    }
}

impl<W: Write> TickRecorder<Statistics> for CsvRecorder<W> {
    type Error = io::Error;

    fn record(&mut self, report: TickReport, output: &Statistics) -> io::Result<()> {
        writeln!(
            self.writer,
            "{},{},{},{}",
            report.tick(),
            output.sheep,
            output.wolves,
            output.grown_grass,
        )
    }
}
