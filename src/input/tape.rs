use std::collections::VecDeque;


pub struct Multi {
	data: VecDeque<super::SampleTape>,
	view: super::SampleTape,
	size: usize,
	channels: usize,
}

impl Multi {
	pub fn new(size: usize, channels: usize) -> Self {
		Multi { size, channels, data: VecDeque::default(), view: super::SampleTape::default() }
	}

	pub fn read(&self) -> super::SampleTape { self.view.clone() }

	pub fn write(&mut self, tape: super::SampleTape) {
		self.data.push_back(tape);
		while self.data.len() > self.size {
			self.data.pop_front();
		}

		let mut joined = vec![vec![]; self.channels];

		// wow i sure love wasting cycles cloning over and over... TODO iter impl
		for s in self.data.iter() {
			for (i, ss) in s.iter().enumerate() {
				joined[i].append(&mut ss.clone());
			}
		}

		self.view = joined;
	}
}
