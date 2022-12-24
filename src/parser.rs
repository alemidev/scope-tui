// use libpulse_binding::sample::Format;

// pub fn parser(fmt: Format) -> impl SampleParser {
// 	match fmt {
// 		Format::S16NE => Signed16PCM {},
// 		_ => panic!("parser not implemented for this format")
// 	}
// }

pub trait SampleParser {
	fn oscilloscope(&self, data: &mut [u8]) -> (Vec<(f64, f64)>, Vec<(f64, f64)>);
	fn vectorscope (&self, data: &mut [u8]) -> Vec<(f64, f64)>;
}

pub struct Signed16PCM {}

impl SampleParser for Signed16PCM {
	fn oscilloscope(&self, data: &mut [u8]) -> (Vec<(f64, f64)>, Vec<(f64, f64)>) { 
		let mut left = Vec::new(); // TODO does left really come first?
		let mut right = Vec::new();
		let mut buf : i16 = 0;
		let mut count : f64 = 0.0;
		let mut flip = false;
		let mut side = false;
		for sample in data {
			if flip {
				buf |= (*sample as i16) << 8;
				if side {
					left.push((count, buf as f64));
				} else {
					right.push((count, buf as f64));
					count += 1.0;
				}
				buf = 0;
				side = !side;
			} else {
				buf |= *sample as i16;
			}
			flip = !flip;
		}
		(left, right)
	}

	fn vectorscope(&self, data: &mut [u8]) -> Vec<(f64, f64)> { 
		let mut out = Vec::new(); // TODO does left really come first?
		let mut buf : i16 = 0;
		let mut flip = false;
		let mut point = None;
		for sample in data {
			if flip {
				buf |= (*sample as i16) << 8;
				if point.is_none() {
					point = Some(buf as f64);
				} else {
					out.push((point.unwrap(), buf as f64));
					point = None;
				}
				buf = 0;
			} else {
				buf |= *sample as i16;
			}
			flip = !flip;
		}
		out
	}
}
