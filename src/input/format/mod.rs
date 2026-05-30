pub trait SampleParser<T> {
	fn parse(data: impl Iterator<Item = u8>) -> impl Iterator<Item = T> {
		data.scan(Vec::with_capacity(2), |chunk, x| {
			chunk.push(x);
			if chunk.len() == Self::size() {
				let val = Self::parse_one(chunk.clone()); //	not sure if this is the correct way to pass &mut
				chunk.clear();

				Some(Some(val))
			} else { Some(None) }
		}).flatten()
	}
	
	fn size() -> usize;	//	TODO: change this for greater flexibility, variable-length encoding BS
	fn parse_one(chunk: Vec<u8>) -> T;
}

pub struct Signed16PCM;
impl SampleParser<f64> for Signed16PCM {
	fn size() -> usize { size_of::<i16>() }
	fn parse_one(chunk: Vec<u8>) -> f64 {
		(chunk[0] as i16 | ((chunk[1] as i16) << 8)) as f64 / 32768.0
	}
}
