pub trait SampleParser<T> {
	fn parse(data: impl Iterator<Item = u8>) -> impl Iterator<Item = T>;
}

pub struct Signed16PCM;
impl SampleParser<f64> for Signed16PCM {
	fn parse(data: impl Iterator<Item = u8>) -> impl Iterator<Item = f64> {
		data.scan(Vec::with_capacity(2), |chunk, x| {
			chunk.push(x);
			if chunk.len() == 2 {
				let val = (chunk[0] as i16 | ((chunk[1] as i16) << 8)) as f64 / 32768.0;
				chunk.clear();

				Some(Some(val))
			} else { Some(None) }
		}).flatten()
	}
}
