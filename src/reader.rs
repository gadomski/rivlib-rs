use {Inclination, Point};
use failure::Error;
use std::path::{Path, PathBuf};
use scanifc::{self, Stream};
use scanlib::{self, Pointcloud};
use std::collections::VecDeque;

const DEFAULT_WANT: u32 = 1024;

/// Reads points and other information from rxp files.
#[derive(Debug)]
pub struct Reader {
    path: PathBuf,
    sync_to_pps: bool,
    want: u32,
}

/// An iterator over rxp points.
#[derive(Debug)]
pub struct Points {
    buffer: VecDeque<Point>,
    stream: Stream,
    want: u32,
}

/// An iterator over inclination readings.
#[derive(Debug)]
pub struct Inclinations {
    buffer: VecDeque<Inclination>,
    pointcloud: Pointcloud,
}

impl Reader {
    /// Creates a new reader for the provided path, with `sync_to_pps` set to false.
    ///
    /// # Examples
    ///
    /// ```
    /// let reader = rivlib::Reader::from_path("data/scan.rxp");
    /// ```
    pub fn from_path<P: AsRef<Path>>(path: P) -> Reader {
        Reader {
            path: path.as_ref().to_path_buf(),
            sync_to_pps: false,
            want: DEFAULT_WANT,
        }
    }

    /// Sets the sync-to-pps value for this reader.
    ///
    /// If true, the reader will filter out all points that aren't synchonized to a
    /// pulse-per-second signal.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut reader = rivlib::Reader::from_path("data/scan.rxp").sync_to_pps(true);
    /// ```
    pub fn sync_to_pps(mut self, sync_to_pps: bool) -> Reader {
        self.sync_to_pps = sync_to_pps;
        self
    }

    /// Sets the number of points wanted for each read of the underlying stream.
    ///
    /// # Examples
    ///
    /// ```
    /// let reader = rivlib::Reader::from_path("data/scan.rxp").want(10);
    /// ```
    pub fn want(mut self, want: u32) -> Reader {
        self.want = want;
        self
    }

    /// Returns an iterator over this reader's points.
    ///
    /// # Examples
    ///
    /// ```
    /// let reader = rivlib::Reader::from_path("data/scan.rxp");
    /// let points = reader.points()
    ///     .unwrap()
    ///     .filter_map(|r| r.ok())
    ///     .collect::<Vec<_>>();
    /// ```
    pub fn points(&self) -> Result<Points, Error> {
        let stream = Stream::open(&self.path, self.sync_to_pps)?;
        Ok(Points {
            buffer: VecDeque::new(),
            stream: stream,
            want: self.want,
        })
    }

    /// Returns an iterator over the inclinations in this rxp file.
    ///
    /// # Examples
    ///
    /// ```
    /// let reader = rivlib::Reader::from_path("data/scan.rxp");
    /// let inclinations = reader.inclinations()
    ///     .unwrap()
    ///     .filter_map(|r| r.ok())
    ///     .collect::<Vec<_>>();
    /// ```
    pub fn inclinations(&self) -> Result<Inclinations, Error> {
        Ok(Inclinations {
            buffer: VecDeque::new(),
            pointcloud: Pointcloud::from_path(&self.path, self.sync_to_pps)?,
        })
    }
}

impl Iterator for Points {
    type Item = Result<Point, scanifc::Error>;

    fn next(&mut self) -> Option<Result<Point, scanifc::Error>> {
        if let Some(point) = self.buffer.pop_front() {
            Some(Ok(point))
        } else {
            match self.stream.read(self.want) {
                Ok(points) => if points.is_empty() {
                    None
                } else {
                    self.buffer.extend(points);
                    self.next()
                },
                Err(err) => Some(Err(err)),
            }
        }
    }
}

impl Iterator for Inclinations {
    type Item = Result<Inclination, scanlib::Error>;

    fn next(&mut self) -> Option<Result<Inclination, scanlib::Error>> {
        // TODO can I refactor this so the iterators can share logic?
        if let Some(inclination) = self.buffer.pop_front() {
            Some(Ok(inclination))
        } else {
            match self.pointcloud.read_inclinations() {
                Ok(option) => if let Some(inclinations) = option {
                    self.buffer.extend(inclinations);
                    self.next()
                } else {
                    None
                },
                Err(err) => Some(Err(err)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reader_from_path() {
        Reader::from_path("data/scan.rxp");
    }

    #[test]
    fn sync_to_pps() {
        Reader::from_path("data/scan.rxp").sync_to_pps(true);
    }
}
