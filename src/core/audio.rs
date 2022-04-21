extern crate cpal;
use cpal::traits::{DeviceTrait, HostTrait};
use crate::core::node::BUFFER_SIZE;

extern crate ringbuf;

pub struct AudioManager {
	host: cpal::Host,
	output_device: cpal::Device,
	output_stream: Option<cpal::Stream>
}

impl AudioManager {
	pub fn new() -> AudioManager {
		let host = cpal::default_host();
		println!("Instantiated audio host with id {:?}", host.id());

		let output_device = host.default_output_device().expect("No output device available!");

		AudioManager {
			host: host,
			output_device: output_device,
			output_stream: None
		}
	}

	pub fn open_output_stream(&mut self, mut ringbuf_consumer: ringbuf::Consumer::<f32>, generator_thread: std::thread::Thread) {
		let supported_config = self.output_device.default_output_config().expect("Default output config not found?");
		println!("Default output config: {:?}", supported_config);

		let supported_channels = supported_config.channels();
		let desired_channels: u16 = 2;

		let supported_sample_rate = supported_config.sample_rate();
		let desired_sample_rate: u32 = 44100;

		let supported_sample_format = supported_config.sample_format();
		let desired_sample_format = cpal::SampleFormat::F32;

		let supported_buffer_size = supported_config.buffer_size();
		let desired_buffer_size: u32 = (BUFFER_SIZE / 2).try_into().unwrap();

		// the final number of samples to generate per stream call
		let output_size: usize = (desired_buffer_size * u32::from(desired_channels)).try_into().unwrap();

		if supported_channels!= desired_channels {
			panic!("Output stream should have {} channels, but the default config instead has {} channels!", desired_channels, supported_channels);
		}

		if supported_sample_rate.0 != desired_sample_rate {
			panic!("Output stream should have a sample rate of {} Hz, but the default config instead has a sample of {} Hz!", desired_sample_rate, supported_sample_rate.0);
		}

		if supported_sample_format != desired_sample_format {
			panic!("Output stream should have a sample format of type {:?}, but the default config instead has a format of type {:?}!", desired_sample_format, supported_sample_format);
		}

		match supported_buffer_size {
			cpal::SupportedBufferSize::Unknown => {
				panic!("Output stream should have a buffer size of {}, but the default config has a unknown buffer size", desired_buffer_size);
			},
			cpal::SupportedBufferSize::Range { min, max } => {
				if &desired_buffer_size < min || &desired_buffer_size > max {
					panic!("Output stream should have a buffer size of {}, but the default config wants a buffer between {} and {}", desired_buffer_size, min, max);
				}
			}
		}

		let mut config = supported_config.config();

		config.buffer_size = cpal::BufferSize::Fixed(desired_buffer_size);

		let error_fn = |err| eprintln!("Error building output sound stream: {}", err);

		let stream_result = self.output_device.build_output_stream(&config, move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
			if output.len() != output_size {
				panic!("output.len() != output_size ({} != {})", output.len(), output_size);
			}

			if ringbuf_consumer.len() < output_size {
				//println!("Not enough samples in ring buffer, sending zeroes instead");
				output.iter_mut().for_each(|m| *m = 0.0);
			} else {
				ringbuf_consumer.pop_slice(output);
				//println!("{} samples left in ring buffer", ringbuf_consumer.len())
			}
			
			generator_thread.unpark();

		}, error_fn);

		match stream_result {
			Ok(stream) => {
				self.output_stream = Some(stream)
			},
			Err(e) => {
				panic!("Error when opening output stream: {}", e)
			}
		}
	}
}