use rodio::{OutputStream, Sink, buffer::SamplesBuffer};

/// # APU
/// The Audio Processing Unit, which manages the sound system.
pub struct APU {
    /// APU Registers 0xFF10-0xFF3F.
    regs: [u8; 0x30],
    /// Internal Wave RAM 0xFF30-0xFF3F.
    wave_ram: [u8; 16],
    /// APU interrupt mask for registers IE and IF.
    pub i_mask: u8,

    /// Audio device.
    sink: Sink,
    /// We must keep the stream alive for audio to play.
    _stream: OutputStream,

    /// Audio buffer.
    buffer: Vec<f32>,

    /// The sample timer.
    sample_timer: f32,
    /// Cycles per sample.
    t_cycles_per_sample: f32,

    /// Frame sequencer.
    frame_sequencer: u8,
    frame_timer: u64,

    // Channel 1.
    ch1_enabled: bool,
    ch1_timer: i32,
    ch1_duty_step: usize,
    ch1_volume: u8,
    ch1_envelope_timer: u8,
    ch1_envelope_running: bool,
    ch1_sweep_timer: u8,
    ch1_sweep_shadow_freq: u16,
    ch1_sweep_enabled: bool,

    // Channel 2.
    ch2_enabled: bool,
    ch2_timer: i32,
    ch2_duty_step: usize,
    ch2_volume: u8,
    ch2_envelope_timer: u8,
    ch2_envelope_running: bool,

    // Channel 3.
    ch3_enabled: bool,
    ch3_timer: i32,
    ch3_sample_idx: usize,

    // Channel 4.
    ch4_enabled: bool,
    ch4_timer: i32,
    ch4_lfsr: u16,
    ch4_volume: u8,
    ch4_envelope_timer: u8,
    ch4_envelope_running: bool,

    // Length Counters.
    ch1_length_timer: u16,
    ch1_length_enabled: bool,
    ch2_length_timer: u16,
    ch2_length_enabled: bool,
    ch3_length_timer: u16,
    ch3_length_enabled: bool,
    ch4_length_timer: u16,
    ch4_length_enabled: bool,

    // Accumulated.
    accumulated_l: f32,
    accumulated_r: f32,
    accumulated_count: u32,
}

impl APU {
    pub fn new() -> Self {
        // Initialize Rodio.
        let stream_handle =
            rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");
        let sink = rodio::Sink::connect_new(stream_handle.mixer());

        Self {
            regs: [0; 0x30],
            wave_ram: [0; 16],
            i_mask: 0,
            sink,
            _stream: stream_handle,
            buffer: Vec::with_capacity(1024),
            sample_timer: 0.0,
            // 4194304 Hz / 44100 Hz = 95.1089...
            t_cycles_per_sample: 4194304.0 / 44100.0,

            frame_sequencer: 0,
            frame_timer: 8192,

            // CH1.
            ch1_enabled: false,
            ch1_timer: 0,
            ch1_duty_step: 0,
            ch1_volume: 0,
            ch1_envelope_timer: 0,
            ch1_envelope_running: false,
            ch1_sweep_timer: 0,
            ch1_sweep_shadow_freq: 0,
            ch1_sweep_enabled: false,
            // CH2.
            ch2_enabled: false,
            ch2_timer: 0,
            ch2_duty_step: 0,
            ch2_volume: 0,
            ch2_envelope_timer: 0,
            ch2_envelope_running: false,
            // CH3.
            ch3_enabled: false,
            ch3_timer: 0,
            ch3_sample_idx: 0,
            // CH4.
            ch4_enabled: false,
            ch4_timer: 0,
            ch4_lfsr: 0,
            ch4_volume: 0,
            ch4_envelope_timer: 0,
            ch4_envelope_running: false,
            // Length counters.
            ch1_length_timer: 0,
            ch1_length_enabled: false,
            ch2_length_timer: 0,
            ch2_length_enabled: false,
            ch3_length_timer: 0,
            ch3_length_enabled: false,
            ch4_length_timer: 0,
            ch4_length_enabled: false,

            accumulated_l: 0.0,
            accumulated_r: 0.0,
            accumulated_count: 0,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF10..=0xFF25 => self.regs[(address - 0xFF10) as usize],
            0xFF26 => {
                let mut val = self.regs[0x16] & 0x80; // Get the Master On/Off bit
                if self.ch1_enabled {
                    val |= 0x01;
                }
                if self.ch2_enabled {
                    val |= 0x02;
                }
                if self.ch3_enabled {
                    val |= 0x04;
                }
                if self.ch4_enabled {
                    val |= 0x08;
                }
                val | 0x70 // Unused bits are usually read as 1
            }
            0xFF30..=0xFF3F => self.wave_ram[(address - 0xFF30) as usize],
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        // Check current state of NR52: Audio master control.
        let master_on = (self.regs[0x16] & 0x80) != 0;

        // If APU is off, ignore writes to $FF10-$FF25.
        if !master_on && (0xFF10..=0xFF25).contains(&address) {
            return;
        }

        // Always store in regs array so read() works.
        if (0xFF10..=0xFF2F).contains(&address) {
            self.regs[(address - 0xFF10) as usize] = value;
        } else if (0xFF30..=0xFF3F).contains(&address) {
            self.wave_ram[(address - 0xFF30) as usize] = value;
        }

        match address {
            // Length registers.
            0xFF11 => self.ch1_length_timer = 64 - (value & 0x3F) as u16,
            0xFF16 => self.ch2_length_timer = 64 - (value & 0x3F) as u16,
            0xFF1B => self.ch3_length_timer = 256 - value as u16,
            0xFF20 => self.ch4_length_timer = 64 - (value & 0x3F) as u16,

            // Triggers.
            0xFF14 => {
                // CH1.
                self.ch1_length_enabled = (value & 0x40) != 0;
                if value & 0x80 != 0 {
                    self.trigger_ch1(value);
                    if self.ch1_length_timer == 0 {
                        self.ch1_length_timer = 64;
                    }
                }
            }
            0xFF19 => {
                // CH2.
                self.ch2_length_enabled = (value & 0x40) != 0;
                if value & 0x80 != 0 {
                    self.trigger_ch2(value);
                    if self.ch2_length_timer == 0 {
                        self.ch2_length_timer = 64;
                    }
                }
            }
            0xFF1E => {
                // CH3.
                self.ch3_length_enabled = (value & 0x40) != 0;
                if value & 0x80 != 0 {
                    self.trigger_ch3(value);
                    if self.ch3_length_timer == 0 {
                        self.ch3_length_timer = 256;
                    }
                }
            }
            0xFF23 => {
                // CH4.
                self.ch4_length_enabled = (value & 0x40) != 0;
                if value & 0x80 != 0 {
                    self.trigger_ch4();
                    if self.ch4_length_timer == 0 {
                        self.ch4_length_timer = 64;
                    }
                }
            }

            0xFF26 => {
                // NR52 - Audio master control.
                // If bit 7 is being toggled OFF.
                if value & 0x80 == 0 {
                    // Clear all registers $FF10-$FF25.
                    for i in 0..0x16 {
                        self.regs[i] = 0;
                    }
                    self.ch1_enabled = false;
                    self.ch2_enabled = false;
                    self.ch3_enabled = false;
                    self.ch4_enabled = false;
                }
            }
            _ => {}
        }
    }

    /// Channel 1 trigger function.
    pub fn trigger_ch1(&mut self, value: u8) {
        self.ch1_enabled = true;
        self.ch1_envelope_running = true;

        // Sweep initialization.
        let nr10 = self.read(0xFF10);
        let sweep_pace = (nr10 & 0x70) >> 4;
        let sweep_step = nr10 & 0x07;

        let freq_low = self.read(0xFF13) as u16;
        let freq_high = (value & 0x07) as u16;
        self.ch1_sweep_shadow_freq = (freq_high << 8) | freq_low;

        self.ch1_sweep_timer = if sweep_pace > 0 { sweep_pace } else { 8 };
        self.ch1_sweep_enabled = sweep_pace > 0 || sweep_step > 0;

        // If step is > 0, overflow check happens immediately on trigger.
        if sweep_step > 0 {
            self.check_sweep_overflow();
        }
        // End sweep init.

        let nr12 = self.read(0xFF12);
        self.ch1_volume = (nr12 & 0xF0) >> 4;
        self.ch1_envelope_timer = nr12 & 0x07;
        self.ch1_duty_step = 0;
        self.ch1_timer = (2048 - self.ch1_sweep_shadow_freq as i32) * 4;
    }

    /// Channel 2 trigger function.
    pub fn trigger_ch2(&mut self, value: u8) {
        self.ch2_enabled = true;
        self.ch2_envelope_running = true;
        let nr22 = self.read(0xFF17);
        self.ch2_volume = (nr22 & 0xF0) >> 4;
        self.ch2_envelope_timer = nr22 & 0x07;
        self.ch2_duty_step = 0;
        let freq = ((value as u16 & 0x07) << 8) | self.read(0xFF18) as u16;
        self.ch2_timer = (2048 - freq as i32) * 4;
    }

    /// Channel 3 trigger function.
    pub fn trigger_ch3(&mut self, value: u8) {
        self.ch3_enabled = true;
        self.ch3_sample_idx = 0;
        let freq = ((value as u16 & 0x07) << 8) | self.read(0xFF1D) as u16;
        self.ch3_timer = (2048 - freq as i32) * 2; // Note: Ch3 timer is *2, not *4
    }

    /// Channel 4 trigger function.
    pub fn trigger_ch4(&mut self) {
        self.ch4_enabled = true;
        self.ch4_envelope_running = true;
        let nr42 = self.read(0xFF21);
        self.ch4_volume = (nr42 & 0xF0) >> 4;
        self.ch4_envelope_timer = nr42 & 0x07;
        // Reset LFSR.
        self.ch4_lfsr = 0x7FFF;
    }

    /// Run the APU for `t_cycles` T-cycles.
    pub fn cycle(&mut self, t_cycles: u64) {
        // --- 1. Update Hardware Timers ---
        // Channel 1
        let freq_low = self.read(0xFF13) as u16;
        let freq_high = (self.read(0xFF14) & 0x07) as u16;
        let frequency = (freq_high << 8) | freq_low;
        let period_ch1 = (2048 - frequency as i32) * 4;

        self.ch1_timer -= t_cycles as i32;
        if self.ch1_timer <= 0 {
            self.ch1_timer += period_ch1;
            self.ch1_duty_step = (self.ch1_duty_step + 1) % 8;
        }

        // Channel 2
        let ch2_freq = (((self.read(0xFF19) & 0x07) as u16) << 8) | self.read(0xFF18) as u16;
        self.ch2_timer -= t_cycles as i32;
        if self.ch2_timer <= 0 {
            self.ch2_timer += (2048 - ch2_freq as i32) * 4;
            self.ch2_duty_step = (self.ch2_duty_step + 1) % 8;
        }

        // Channel 3
        let ch3_freq = (((self.read(0xFF1E) & 0x07) as u16) << 8) | self.read(0xFF1D) as u16;
        self.ch3_timer -= t_cycles as i32;
        if self.ch3_timer <= 0 {
            self.ch3_timer += (2048 - ch3_freq as i32) * 2;
            self.ch3_sample_idx = (self.ch3_sample_idx + 1) % 32;
        }

        // Channel 4
        let nr43 = self.read(0xFF22);
        let shift = (nr43 >> 4) as i32;
        let divisor = match nr43 & 0x07 {
            0 => 8,
            1 => 16,
            2 => 32,
            3 => 48,
            4 => 64,
            5 => 80,
            6 => 96,
            7 => 112,
            _ => 8,
        };
        let period_ch4 = divisor << shift;

        self.ch4_timer -= t_cycles as i32;
        if self.ch4_timer <= 0 {
            self.ch4_timer += period_ch4;
            let bit0 = self.ch4_lfsr & 0x01;
            let bit1 = (self.ch4_lfsr >> 1) & 0x01;
            let result = bit0 ^ bit1;
            self.ch4_lfsr = (self.ch4_lfsr >> 1) | (result << 14);
            if (nr43 & 0x08) != 0 {
                self.ch4_lfsr = (self.ch4_lfsr & !(1 << 6)) | (result << 6);
            }
        }

        // --- 2. Frame Sequencer (Length, Sweep, Envelope) ---
        self.frame_timer += t_cycles;
        if self.frame_timer >= 8192 {
            self.frame_timer -= 8192;
            match self.frame_sequencer {
                0 | 2 | 4 | 6 => {
                    self.step_length();
                    if self.frame_sequencer == 2 || self.frame_sequencer == 6 {
                        self.step_sweep();
                    }
                }
                7 => {
                    self.step_envelope_ch1();
                    self.step_envelope_ch2();
                    self.step_envelope_ch4();
                }
                _ => {}
            }
            self.frame_sequencer = (self.frame_sequencer + 1) % 8;
        }

        // --- 3. Anti-Aliasing Accumulation ---
        // We capture the state of the audio for these cycles and add it to our average
        let (l_sample, r_sample) = self.generate_sample();
        self.accumulated_l += l_sample * t_cycles as f32;
        self.accumulated_r += r_sample * t_cycles as f32;
        self.accumulated_count += t_cycles as u32;

        // --- 4. Sample Generation & Resampling ---
        self.sample_timer += t_cycles as f32;
        while self.sample_timer >= self.t_cycles_per_sample {
            self.sample_timer -= self.t_cycles_per_sample;

            // Calculate average for this sample period to smooth the sound
            if self.accumulated_count > 0 {
                let avg_l = self.accumulated_l / self.accumulated_count as f32;
                let avg_r = self.accumulated_r / self.accumulated_count as f32;
                self.buffer.push(avg_l);
                self.buffer.push(avg_r);

                // Reset accumulators for next sample
                self.accumulated_l = 0.0;
                self.accumulated_r = 0.0;
                self.accumulated_count = 0;
            }

            // --- 5. Buffer Management & Throttling ---
            if self.buffer.len() >= 1024 {
                // Increased sink limit slightly to prevent underflow "tearing"
                if self.sink.len() < 10 {
                    let source = rodio::buffer::SamplesBuffer::new(2, 44100, self.buffer.clone());
                    self.sink.append(source);
                }
                self.buffer.clear();
            }
        }
    }

    fn step_length(&mut self) {
        // Channel 1.
        if self.ch1_length_enabled && self.ch1_length_timer > 0 {
            self.ch1_length_timer -= 1;
            if self.ch1_length_timer == 0 {
                self.ch1_enabled = false;
            }
        }
        // Channel 2.
        if self.ch2_length_enabled && self.ch2_length_timer > 0 {
            self.ch2_length_timer -= 1;
            if self.ch2_length_timer == 0 {
                self.ch2_enabled = false;
            }
        }
        // Channel 3.
        if self.ch3_length_enabled && self.ch3_length_timer > 0 {
            self.ch3_length_timer -= 1;
            if self.ch3_length_timer == 0 {
                self.ch3_enabled = false;
            }
        }
        // Channel 4.
        if self.ch4_length_enabled && self.ch4_length_timer > 0 {
            self.ch4_length_timer -= 1;
            if self.ch4_length_timer == 0 {
                self.ch4_enabled = false;
            }
        }
    }

    /// Channel 1 envelope step.
    fn step_envelope_ch1(&mut self) {
        let nr12 = self.read(0xFF12);
        let sweep_pace = nr12 & 0x07;

        // If pace is 0, the envelope is disabled.
        if sweep_pace == 0 {
            return;
        }

        if self.ch1_envelope_running && self.ch1_envelope_timer > 0 {
            self.ch1_envelope_timer -= 1;

            if self.ch1_envelope_timer == 0 {
                self.ch1_envelope_timer = sweep_pace;

                // True = up, False = down.
                let direction = (nr12 & 0x08) != 0;
                if direction && self.ch1_volume < 15 {
                    self.ch1_volume += 1;
                } else if !direction && self.ch1_volume > 0 {
                    self.ch1_volume -= 1;
                }

                if self.ch1_volume == 0 || self.ch1_volume == 15 {
                    self.ch1_envelope_running = false;
                }
            }
        }
    }
    /// Channel 2 envelope step.
    fn step_envelope_ch2(&mut self) {
        let nr22 = self.read(0xFF17);
        let sweep_pace = nr22 & 0x07;

        // If pace is 0, the envelope is disabled.
        if sweep_pace == 0 {
            return;
        }

        if self.ch2_envelope_running && self.ch2_envelope_timer > 0 {
            self.ch2_envelope_timer -= 1;

            if self.ch2_envelope_timer == 0 {
                self.ch2_envelope_timer = sweep_pace;

                // True = up, False = down.
                let direction = (nr22 & 0x08) != 0;
                if direction && self.ch2_volume < 15 {
                    self.ch2_volume += 1;
                } else if !direction && self.ch2_volume > 0 {
                    self.ch2_volume -= 1;
                }

                if self.ch2_volume == 0 || self.ch2_volume == 15 {
                    self.ch2_envelope_running = false;
                }
            }
        }
    }

    /// Channel 4 envelope step.
    fn step_envelope_ch4(&mut self) {
        let nr42 = self.read(0xFF21);
        let sweep_pace = nr42 & 0x07;

        // If pace is 0, the envelope is disabled.
        if sweep_pace == 0 {
            return;
        }

        if self.ch4_envelope_running && self.ch4_envelope_timer > 0 {
            self.ch4_envelope_timer -= 1;

            if self.ch4_envelope_timer == 0 {
                self.ch4_envelope_timer = sweep_pace;

                // True = up, False = down.
                let direction = (nr42 & 0x08) != 0;
                if direction && self.ch4_volume < 15 {
                    self.ch4_volume += 1;
                } else if !direction && self.ch4_volume > 0 {
                    self.ch4_volume -= 1;
                }

                if self.ch4_volume == 0 || self.ch4_volume == 15 {
                    self.ch4_envelope_running = false;
                }
            }
        }
    }

    /// Generates a stereo sample from the channels.
    fn generate_sample(&self) -> (f32, f32) {
        // Bit 7 of NR52 (0xFF26) is the Master Sound on/off switch.
        let master_on = (self.read(0xFF26) & 0x80) != 0;
        if !master_on {
            return (0.0, 0.0);
        }

        let ch1 = self.calculate_ch1();
        let ch2 = self.calculate_ch2();
        let ch3 = self.calculate_ch3();
        let ch4 = self.calculate_ch4();

        let nr51 = self.read(0xFF25);
        let mut left = 0.0;
        let mut right = 0.0;

        // Right Channel Panning.
        if nr51 & 0x01 != 0 {
            right += ch1;
        }
        if nr51 & 0x02 != 0 {
            right += ch2;
        }
        if nr51 & 0x04 != 0 {
            right += ch3;
        }
        if nr51 & 0x08 != 0 {
            right += ch4;
        }

        // Left Channel Panning.
        if nr51 & 0x10 != 0 {
            left += ch1;
        }
        if nr51 & 0x20 != 0 {
            left += ch2;
        }
        if nr51 & 0x40 != 0 {
            left += ch3;
        }
        if nr51 & 0x80 != 0 {
            left += ch4;
        }

        // Master Volume (NR50).
        let nr50 = self.read(0xFF24);
        let r_vol = ((nr50 & 0x07) as f32 + 1.0) / 8.0;
        let l_vol = (((nr50 & 0x70) >> 4) as f32 + 1.0) / 8.0;

        // Apply master volume and a small safety gain to prevent digital clipping
        (left * l_vol * 0.2, right * r_vol * 0.2)
    }

    fn calculate_ch1(&self) -> f32 {
        if !self.ch1_enabled || self.ch1_volume == 0 {
            return 0.0;
        }

        let duty_idx = (self.read(0xFF11) >> 6) as usize;
        let patterns = [
            [0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 1, 1, 1],
            [0, 1, 1, 1, 1, 1, 1, 0],
        ];

        let signal = patterns[duty_idx][self.ch1_duty_step];

        // Scale by volume (0.0 to 1.0 range).
        // We use a small multiplier (0.05) so it's not too loud.
        let volume_multiplier = (self.ch1_volume as f32 / 15.0) * 0.05;

        if signal == 1 {
            volume_multiplier
        } else {
            -volume_multiplier
        }
    }

    fn calculate_ch2(&self) -> f32 {
        if !self.ch2_enabled || self.ch2_volume == 0 {
            return 0.0;
        }

        let duty_idx = (self.read(0xFF16) >> 6) as usize;
        let patterns = [
            [0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 0, 1, 1, 1],
            [0, 1, 1, 1, 1, 1, 1, 0],
        ];

        let signal = patterns[duty_idx][self.ch2_duty_step];

        // Scale by volume (0.0 to 1.0 range).
        // We use a small multiplier (0.05) so it's not too loud.
        let volume_multiplier = (self.ch2_volume as f32 / 15.0) * 0.05;

        if signal == 1 {
            volume_multiplier
        } else {
            -volume_multiplier
        }
    }

    fn calculate_ch3(&self) -> f32 {
        let nr30 = self.read(0xFF1A);
        if !self.ch3_enabled || (nr30 & 0x80 == 0) {
            return 0.0;
        }

        // Volume shift: bits 5-6 of NR32 ($FF1C)
        let volume_shift = match (self.read(0xFF1C) >> 5) & 0x03 {
            0 => 4, // Mute (actually shifts out all bits)
            1 => 0, // 100%
            2 => 1, // 50%
            3 => 2, // 25%
            _ => 4,
        };

        // Get 4-bit sample from Wave RAM (32 samples total, 2 per byte)
        let byte = self.wave_ram[self.ch3_sample_idx / 2];
        let mut sample = if self.ch3_sample_idx.is_multiple_of(2) {
            byte >> 4 // High nibble
        } else {
            byte & 0x0F // Low nibble
        };

        sample >>= volume_shift;

        // Normalize 0..15 to -1.0..1.0
        (sample as f32 / 7.5 - 1.0) * 0.05
    }

    fn calculate_ch4(&self) -> f32 {
        if !self.ch4_enabled || self.ch4_volume == 0 {
            return 0.0;
        }

        // Result is the inverse of the first bit
        let bit = (!self.ch4_lfsr) & 0x01;
        let vol = (self.ch4_volume as f32 / 15.0) * 0.05;

        if bit == 1 { vol } else { -vol }
    }

    fn step_sweep(&mut self) {
        if !self.ch1_sweep_enabled {
            return;
        }

        if self.ch1_sweep_timer > 0 {
            self.ch1_sweep_timer -= 1;
        }

        if self.ch1_sweep_timer == 0 {
            let nr10 = self.read(0xFF10);
            let sweep_pace = (nr10 & 0x70) >> 4;
            self.ch1_sweep_timer = if sweep_pace > 0 { sweep_pace } else { 8 };

            if sweep_pace > 0 {
                let new_freq = self.calculate_sweep_freq();

                let sweep_step = nr10 & 0x07;
                if new_freq <= 2047 && sweep_step > 0 {
                    // Update shadow frequency
                    self.ch1_sweep_shadow_freq = new_freq;

                    // Update NR13 and NR14 registers
                    self.regs[0x03] = (new_freq & 0xFF) as u8;
                    self.regs[0x04] = (self.regs[0x04] & 0xF8) | ((new_freq >> 8) & 0x07) as u8;

                    // Overflow check again with the NEW frequency
                    self.calculate_sweep_freq();
                }
            }
        }
    }

    fn calculate_sweep_freq(&mut self) -> u16 {
        let nr10 = self.read(0xFF10);
        let sweep_step = nr10 & 0x07;
        let descending = (nr10 & 0x08) != 0;

        let delta = self.ch1_sweep_shadow_freq >> sweep_step;
        let new_freq = if descending {
            self.ch1_sweep_shadow_freq.saturating_sub(delta)
        } else {
            self.ch1_sweep_shadow_freq + delta
        };

        // Overflow check.
        if new_freq > 2047 {
            self.ch1_enabled = false;
        }

        new_freq
    }

    fn check_sweep_overflow(&mut self) {
        self.calculate_sweep_freq();
    }

    /// Flush the buffer.
    pub fn flush(&mut self) {
        if !self.buffer.is_empty() {
            let source = SamplesBuffer::new(2, 44100, self.buffer.clone());
            self.sink.append(source);
            self.buffer.clear();
        }
    }
}
