// use libpulse_binding::sample::Format;

// pub fn parser(fmt: Format) -> impl SampleParser {
// 	match fmt {
// 		Format::S16NE => Signed16PCM {},
// 		_ => panic!("parser not implemented for this format")
// 	}
// }

pub trait SampleParser {
	fn oscilloscope(&self, data: &mut [u8], channels: u32) -> Vec<Vec<f64>>;
}

pub struct Signed16PCM {}

/// TODO these are kinda inefficient, can they be faster?
impl SampleParser for Signed16PCM {
	fn oscilloscope(&self, data: &mut [u8], channels: u32) -> Vec<Vec<f64>> {
		let mut out = vec![vec![]; channels as usize];
		let mut channel = 0;
		for chunk in data.chunks(2) {
			let buf = chunk[0] as i16 | (chunk[1] as i16) << 8;
			out[channel].push(buf as f64);
			channel = (channel + 1 ) % channels as usize;
		}
		out
	}
}
