use std::{str::FromStr, num::ParseIntError};

#[derive(Debug, PartialEq, Clone)]
pub enum Tone {
	C, Db, D, Eb, E, F, Gb, G, Ab, A, Bb, B
}

pub struct ToneError {}

#[derive(Debug, PartialEq, Clone)]
pub struct Note {
	tone: Tone,
	octave: u32,
}

pub enum NoteError {
	InvalidOctave(ParseIntError),
	InalidNote(ToneError),
}

impl From::<ToneError> for NoteError {
	fn from(e: ToneError) -> Self {
		NoteError::InalidNote(e)
	}
}

impl From::<ParseIntError> for NoteError {
	fn from(e: ParseIntError) -> Self {
		NoteError::InvalidOctave(e)
	}
}

impl FromStr for Note {
	type Err = NoteError;

	fn from_str(txt: &str) -> Result<Self, Self::Err> {
		let trimmed = txt.trim();
		let mut split = 0;
		for c in trimmed.chars() {
			if !c.is_ascii_digit() {
				split += 1;
			} else {
				break;
			}
		}
		Ok(
			Note {
				tone: trimmed[..split].parse::<Tone>()?,
				octave: trimmed[split..].parse::<u32>().unwrap_or(0),
			}
		)
	}
}

// impl TryFrom::<String> for Note {
// 	type Error = NoteError;
// 	fn try_from(value: String) -> Result<Self, Self::Error> {
// 		value.as_str().parse::<Note>()
// 	}
// }

impl FromStr for Tone {
	type Err = ToneError;

	fn from_str(txt: &str) -> Result<Self, Self::Err> {
		match txt {
			"C"         => Ok(Tone::C ),
			"C#" | "Db" => Ok(Tone::Db),
			"D"         => Ok(Tone::D ),
			"D#" | "Eb" => Ok(Tone::Eb),
			"E"         => Ok(Tone::E ),
			"F"         => Ok(Tone::F ),
			"F#" | "Gb" => Ok(Tone::Gb),
			"G"         => Ok(Tone::G ),
			"G#" | "Ab" => Ok(Tone::Ab),
			"A"         => Ok(Tone::A ),
			"A#" | "Bb" => Ok(Tone::Bb),
			"B"         => Ok(Tone::B ),
			_           => Err(ToneError {  })
		}
	}
}

impl Note {
	pub fn tune_buffer_size(&self, sample_rate: u32) -> u32 {
		let t = 1.0 / self.tone.freq(self.octave); // periodo ?
		let buf = (sample_rate as f32) * t;
		return (buf * 4.0).round() as u32;
	}

	// pub fn tune_sample_rate(&self, octave:u32, buffer_size: u32) -> u32 {
	// 	// TODO does it just work the same way?
	// 	let t = 1.0 / self.freq(octave); // periodo ?
	// 	let buf = (buffer_size as f32) * t;
	// 	return (buf * 4.0).round() as u32;
	// }
}

impl Tone {
	pub fn freq(&self, octave: u32) -> f32 {
		match octave {
			0 => match self {
				Tone::C  => 16.35,
				Tone::Db => 17.32,
				Tone::D  => 18.35,
				Tone::Eb => 19.45,
				Tone::E  => 20.60,
				Tone::F  => 21.83,
				Tone::Gb => 23.12,
				Tone::G  => 24.50,
				Tone::Ab => 25.96,
				Tone::A  => 27.50,
				Tone::Bb => 29.14,
				Tone::B  => 30.87,
			},
			_ => {
				let mut freq = self.freq(0);
				for _ in 0..octave {
					freq *= 2.0;
				}
				freq
			}
		}
	}

	// pub fn all() -> Vec<Note> {
	// 	vec![Note::C, Note::Db, Note::D, Note::Eb, Note::E, Note::F, Note::Gb, Note::G, Note::Ab, Note::A, Note::Bb, Note::B]
	// }
}

