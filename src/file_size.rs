use std::fmt;

/// Pretty file size formatter.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, dataview::Pod)]
#[repr(transparent)]
pub struct FileSize(pub u64);

impl fmt::Display for FileSize {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if self.0 < 1024 {
			let unit = if self.0 == 1 { " byte" } else { " bytes" };
			write!(f, "{}{}", self.0, unit)
		}
		else {
			let (size, unit);
			if self.0 < 1024 * 1024 {
				size = self.0 as f64 / 1024.0;
				unit = " KiB";
			}
			else if self.0 < 1024 * 1024 * 1024 {
				size = self.0 as f64 / (1024.0 * 1024.0);
				unit = " MiB";
			}
			else if self.0 < 1024 * 1024 * 1024 * 1024 {
				size = self.0 as f64 / (1024.0 * 1024.0 * 1024.0);
				unit = " GiB";
			}
			else {
				size = self.0 as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0);
				unit = " TiB";
			}
			write!(f, "{:.2}{}", size, unit)
		}
	}
}
