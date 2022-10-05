use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use cpal::StreamConfig;
use std::{thread, time};
use ringbuf::RingBuffer;
use rustfft::{FftPlanner, num_complex::Complex};
use futures::Stream;

use crate::note::Note;

pub struct Microphone {
	device: cpal::Device,
	stream: cpal::Stream,
	config: StreamConfig,
	consumer: ringbuf::Consumer<f32>,
	window_length: time::Duration,
	num_samples_processed: u128,
	elapsed_time: time::Duration,
	fft_planner: FftPlanner<f32>,
}

impl Microphone {
	pub fn new(device: cpal::Device) -> Microphone {
		let mut configs_range = device.supported_input_configs()
			.expect("No supported configs");
		let config = configs_range.next()
			.expect("No supported config");
		let config = config.with_sample_rate(cpal::SampleRate{0: 8000});
		let config = StreamConfig::from(config);

		let buffer_size: usize = config.sample_rate.0.try_into().unwrap();
		//let buffer_size = buffer_size * 2;
		let buffer = RingBuffer::<f32>::new(buffer_size);
		let (mut producer, consumer) = buffer.split();

		let stream = device.build_input_stream(
			&config,
			move |data: &[f32], _: &cpal::InputCallbackInfo| {
				let num = producer.push_slice(data);
				if num < data.len() {
					eprintln!("Not processing audio fast enough");
				}
			},
			move |_| {
			},
		).unwrap();

		Microphone {
			device,
			stream,
			config,
			consumer,
			window_length: time::Duration::from_millis(100),
			num_samples_processed: 0,
			elapsed_time: time::Duration::ZERO,
			fft_planner: rustfft::FftPlanner::new(),
		}
	}

	pub fn play(&self) {
		self.stream.play().unwrap();
	}

	pub fn pause(&self) {
		self.stream.pause().unwrap();
	}

	fn needed_samples(&self) -> usize {
		let rate = self.config.sample_rate.0 as u128;
		let should_have_processed = (self.elapsed_time + self.window_length).as_millis() * rate / 1000;
	    (should_have_processed - self.num_samples_processed) as usize
	}

	pub fn consume(&mut self) -> Option<Note> {
		if !self.ready() {
			None
		} else {
		    let needed_samples = self.needed_samples();
			let buffer_len = needed_samples.try_into().unwrap();

			let mut samples = vec!(0.0; buffer_len);
			self.consumer.pop_slice(&mut samples);

			let freq = self.frequency(&mut samples);
			let note = (12.0 * (freq / 440.0).log2()).floor() as i32;
			let note = note as i8;

			self.num_samples_processed += needed_samples as u128;
			self.elapsed_time += self.window_length;
			let length = 1000.0 * samples.len() as f32 / self.config.sample_rate.0 as f32;

			Some(Note::new(self.window_length.as_millis() as u32, note, true, "".to_string()))
		}
	}

	pub fn clear(&mut self) {
		self.consumer.pop_each(|_| {
			true
		}, None);
	}

	pub fn set_window_length(&mut self, window_length: time::Duration) {
		self.window_length = window_length;
	}

	fn zero_crossing_rate(&self, v: &Vec<f32>) -> f32 {
		let mut zero_cross_rate = 0.0;
		let mut prev = v.get(0).unwrap().clone();
		for elem in v {
			if elem < &0.0 && prev >= 0.0 || elem >= &0.0 && prev < 0.0 {
				zero_cross_rate += 1.0;
			}
			prev = *elem;
		}
		zero_cross_rate /= (v.len() - 1) as f32;
		zero_cross_rate
	}

	fn autocorrelate(&mut self, v: &mut Vec<f32>) -> Vec<f32> {
		//let buffer_len = (2.0 * v.len() as f32).log2().ceil() as u32;
		//let buffer_len = 2_usize.pow(buffer_len);
		let buffer_len = 2 * v.len();
		let fft = self.fft_planner.plan_fft_forward(buffer_len);
		let ffti = self.fft_planner.plan_fft_inverse(buffer_len);

		// zero pad for linear, not circular conv
		v.resize(buffer_len, 0.0);

		// forward transform
		let mut complex = v.iter().map(|&elem| {
			Complex::new(elem, 0.0)
		}).collect::<Vec<Complex<f32>>>();
		fft.process(&mut complex);

		// square
		let mut complex = complex.iter().map(|&elem| {
			Complex::new(elem.norm_sqr(), 0.0)
		}).collect::<Vec<Complex<f32>>>();

		//inverse transform
		ffti.process(&mut complex);
		let mut auto = complex.iter().map(|&elem| {
			elem.re
		}).collect::<Vec<f32>>();

		//ignore duplicate data
		auto.resize(buffer_len, 0.0);

		auto
	}

	fn frequency(&mut self, samples: &mut Vec<f32>) -> f32 {
		let buffer_len = samples.len();
		
		let auto = self.autocorrelate(samples);

		//normalize
		let variance = auto.get(0).unwrap();
		let auto = auto.iter().map(|&elem| {
			elem / variance
		}).collect::<Vec<f32>>();

		let zero_cross_rate = self.zero_crossing_rate(&auto);

		//first zero cross
		let zero_cross = match auto.iter().position(|&elem| {
			elem < 0.0
		}) {
			None => 0,
			Some(z) => z,
		};

		//find max
		let mut max_index: usize = zero_cross;
		for lag in zero_cross..buffer_len {
			if auto.get(lag) > auto.get(max_index) || max_index == 0 {
				max_index = lag;
			}
		}
		
		//convert to frequency
		let peak = auto.get(max_index).unwrap();
		let rate = self.config.sample_rate.0 as f32;
		let freq = rate / max_index as f32;

		freq
	}

	pub fn ready(&self) -> bool {
		let needed_samples = self.needed_samples();
		let available: u128 = self.consumer.len().try_into().unwrap();
		available >= needed_samples as u128
	}
}

impl Stream for Microphone {
    type Item = Note;
    fn poll_next(mut self: std::pin::Pin<&mut Self>, 
                 cx: &mut std::task::Context<'_>
                ) -> std::task::Poll<Option<Self::Item>> {
       if self.ready() {
            std::task::Poll::Ready(self.consume())
       } else {
            std::task::Poll::Pending
       }
    }

}

