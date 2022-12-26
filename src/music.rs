#[derive(Debug, PartialEq, Clone)]
pub enum Note {
	C, Db, D, Eb, E, F, Gb, G, Ab, A, Bb, B,
	INVALID
}

impl From::<String> for Note {
	fn from(txt: String) -> Self {
		match txt.as_str() {
			"C"  => Note::C,
			"Db" => Note::Db,
			"D"  => Note::D,
			"Eb" => Note::Eb,
			"E"  => Note::E,
			"F"  => Note::F,
			"Gb" => Note::Gb,
			"G"  => Note::G,
			"Ab" => Note::Ab,
			"A"  => Note::A,
			"Bb" => Note::Bb,
			"B"  => Note::B,
			_    => Note::INVALID,
		}
	}
}

impl Note {
	pub fn freq(&self, octave: u32) -> f32 {
		match octave {
			0 => match self {
				Note::C  => 16.35,
				Note::Db => 17.32,
				Note::D  => 18.35,
				Note::Eb => 19.45,
				Note::E  => 20.60,
				Note::F  => 21.83,
				Note::Gb => 23.12,
				Note::G  => 24.50,
				Note::Ab => 25.96,
				Note::A  => 27.50,
				Note::Bb => 29.14,
				Note::B  => 30.87,
				Note::INVALID => 0.0,
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

	pub fn tune_buffer_size(&self, octave:u32, sample_rate: u32) -> u32 {
		let t = 1.0 / self.freq(octave); // periodo ?
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

