pub trait SampleParser<T> {
	fn parse(data: &[u8]) -> T;
}

pub struct Signed16PCM;
impl SampleParser<f64> for Signed16PCM {
	fn parse(chunk: &[u8]) -> f64 {
		(chunk[0] as i16 | ((chunk[1] as i16) << 8)) as f64
	}
}
