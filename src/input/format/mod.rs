use std::iter::from_fn;
pub trait SampleParser<T> {	
	const STATIC_SIZE: Option<usize> = None;
	fn parse_next(data: &mut impl Iterator<Item = u8>) -> Option<T>;
	
	fn parse(data: impl Iterator<Item = u8>) -> impl Iterator<Item = T> where 
		Self: Sized,
	{
		let mut source = data;
		from_fn(move || Self::parse_next(&mut source))
	}
}

pub struct Signed16PCM;
impl SampleParser<f64> for Signed16PCM {
	const STATIC_SIZE: Option<usize> = Some(std::mem::size_of::<i16>());
	fn parse_next(data: &mut impl Iterator<Item = u8>) -> Option<f64> {
		let b0 = data.next()?;
		let b1 = data.next()?;
		let raw = (b0 as u16 | ((b1 as u16) << 8)) as i16;
		Some(raw as f64 / 32768.0)
	}
}
