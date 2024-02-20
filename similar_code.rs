use std::process::exit;

use crate::kmath::*;
use crate::time_queue::*;
use crate::sound::*;

const fifth: f32 = 3./2.;
const fourth: f32 = 4. / 3.;
const octave: f32 = 2.;
const minor_third: f32 = 6./5.;
const major_third: f32 = 5./4.;
const unity: f32 = 1.;
const major_seventh: f32 = 17./9.;
const minor_seventh: f32 = 7. / 4.;
const major_sixth: f32 = 27. / 16.;
const minor_sixth: f32 = 8. / 5.;
const semitone: f32 = 1.05946309436;
const tritone: f32 = 729./512.;

pub struct Generator {
    seed: u32,
    stack: Vec<[Sound;4]>,
    unwind: bool,
}

impl Generator {
    pub fn new(seed: u32) -> Generator {
        let mut curr_sound = Sound::new();
        curr_sound.A = 0.05;
        curr_sound.S = 0.0;
        curr_sound.R = 0.5;
        let init_section = [
            curr_sound,
            curr_sound.but(|s| s.freq *= major_third),
            curr_sound.but(|s| s.freq *= fifth),
            curr_sound.but(|s| s.freq *= octave),
        ];
        Generator { seed, stack: vec![init_section], unwind: false }
    }

    // unwind operator
    // does full unwind, changes the base one, to latest one
    // 

    pub fn mutate(&mut self) -> [Sound;4] {
        if self.unwind {
            if self.stack.len() > 1 {
                return self.stack.pop().unwrap();
            } else {
                return [
                    Sound::new().but(|s| s.amplitude = 0.0),
                    Sound::new().but(|s| s.amplitude = 0.0),
                    Sound::new().but(|s| s.amplitude = 0.0),
                    Sound::new().but(|s| s.amplitude = 0.0),
                ]
            }
        }

        if self.stack.len() > 5 && chance(self.seed * 2394872343, 0.05) {
            self.unwind = true;
        } 
    
        let mut section = self.stack[self.stack.len() - 1];
        [
            |i, section: &mut [Sound;4]|{
                let change_idx = (khash(i * 23984737) % 4) as usize;
                let mut k = 0;
                let mut swap_to = 0;
                loop {
                    swap_to = (khash(k * i * 23094823) % 4) as usize;
                    if swap_to != change_idx {
                        break;
                    }
                    k += 1;
                }
                let tmp = section[change_idx];
                section[change_idx] = section[swap_to];
                section[swap_to] = tmp;
            },
            
            |i, section: &mut [Sound;4]|{
                let change_idx = (khash(i) % 4) as usize;
                let change_interval = [minor_third, major_third, fifth, octave, unity][(khash(i * 239847123) % 5) as usize];
                if section[change_idx].freq < 100. || (chance(i * 3194719879, 0.5) && section[change_idx].freq < 1000.) {
                    section[change_idx].freq *= change_interval;
                } else {
                    section[change_idx].freq /= change_interval;
                }
            },

            |i, section: &mut [Sound;4]|{
                let change_interval = [minor_third, major_third, fifth, octave, unity][(khash(i * 239847123) % 5) as usize];
                if section[0].freq < 100. || (chance(i * 3194719879, 0.5) && section[0].freq < 1000.) {
                    section[0].freq *= change_interval;
                    section[1].freq *= change_interval;
                    section[2].freq *= change_interval;
                    section[3].freq *= change_interval;
                } else {
                    section[0].freq /= change_interval;
                    section[1].freq /= change_interval;
                    section[2].freq /= change_interval;
                    section[3].freq /= change_interval;
                }
            },
            
            // |i, section: &mut [Sound;4]|{
            //     let tmp = section[0];
            //     section[0] = section[1];
            //     section[1] = section[2];
            //     section[2] = section[3];
            //     section[3] = tmp;
            // },

            // |i, section: &mut [Sound;4]|{
            //     let tmp = section[0];
            //     section[0] = section[3];
            //     section[3] = section[2];
            //     section[2] = section[1];
            //     section[1] = tmp;
            // },

            |i, section: &mut [Sound;4]|{
                let change_idx = (khash(i) % 4) as usize;
                if section[change_idx].amplitude == 0.0 {
                    section[change_idx].amplitude = 0.1;
                } else {
                    section[change_idx].amplitude = 0.0;
                }
            },

        ][(khash(self.seed * 23423479) % 4) as usize](self.seed * 34902389, &mut section);
        self.stack.push(section);

        self.seed = khash(self.seed);
        return section;

        // pop if not already at start and probability
        // otherwise push new stuff (what we have)

    }
}

pub fn first(seed: u32) -> TimeQueue<Sound> {
    let mut music = TimeQueue::new();

    let mut gen = Generator::new(seed);    

    let mut t = 0.0;
    for i in 0..100 {
        
        let speed = 1.1;
        
        let section = gen.mutate();
        if gen.unwind {
            let ins_t = t; // kuniform(seed * 91283471, -0.05, 0.05) + t;
            music.insert(ins_t, section[0]);
            music.insert(ins_t + 0.25/speed, section[1]);
            music.insert(ins_t + 0.5/speed, section[2]);
            music.insert(ins_t + 0.75/speed, section[3]);
            t += 1.0/speed;
        } else {
            for j in 0..1 {
                let ins_t = t; // kuniform(seed * 91283471, -0.05, 0.05) + t;
                music.insert(ins_t, section[0]);
                music.insert(ins_t + 0.25/speed, section[1]);
                music.insert(ins_t + 0.5/speed, section[2]);
                music.insert(ins_t + 0.75/speed, section[3]);
                t += 1.0/speed;
            }
        }
    }

    music
}

// this is so cool, why not 3
// also could have a lazy evaluation if it maybe a function of (t, sound, seed) -> (t, sound) or something
pub fn sequences(seed: u32) -> TimeQueue<Sound> {
    let mut music = TimeQueue::new();

    let mut s1 = Sound::new();
    s1.A = 0.05;
    s1.S = 0.0;
    s1.R = 0.5;

    let mut s2 = s1.clone();

    let mut t = 0.0;

    for i in 0..200 {
        let selected_interval = [minor_third, major_third, fifth, octave, unity][(khash(seed * 3948723 + i * 239847123) % 5) as usize];
        if i % 2 == 0 {
            if s1.freq < 100. || (chance(seed * 123048132 + i * 3194719879, 0.5) && s1.freq < 1000.) {
               s1.freq *= selected_interval;
            } else {
               s1.freq /= selected_interval;
            }
            music.insert(t, s1);
        } else {
            if s2.freq < 100. || (chance(seed * 123048132 + i * 3194719879, 0.5) && s2.freq < 1000.) {
                s2.freq *= selected_interval;
            } else {
                s2.freq /= selected_interval;
            }
            music.insert(t, s2);
        }
        t += 0.25;
    }

    music
}

pub fn sequences3(seed: u32) -> TimeQueue<Sound> {
    let mut music = TimeQueue::new();

    let mut s1 = Sound::new();
    s1.A = 0.05;
    s1.S = 0.0;
    s1.R = 0.5;

    s1.amp_lfo_amount = 1.0;
    s1.amp_lfo_freq = 10.0;
    
    let mut s2 = s1.clone();
    s2.amp_lfo_freq = 5.0;
    
    let mut s3 = s1.clone();
    s3.amp_lfo_freq = 15.0;

    let mut t = 0.0;

    for i in 0..200 {
        let selected_interval = [minor_third, major_third, fifth, octave, unity][(khash(seed * 3948723 + i * 239847123) % 5) as usize];
        // let selected_interval = tritone;
        // let selected_interval = [minor_third, semitone, tritone][(khash(seed * 3948723 + i * 239847123) % 3) as usize];
        if i % 3 == 0 {
            if s1.freq < 100. || (chance(seed * 123048132 + i * 3194719879, 0.5) && s1.freq < 1000.) {
               s1.freq *= selected_interval;
            } else {
               s1.freq /= selected_interval;
            }
            music.insert(t, s1);
        } else if i % 3 == 1 {
            if s2.freq < 100. || (chance(seed * 123048132 + i * 3194719879, 0.5) && s2.freq < 1000.) {
                s2.freq *= selected_interval;
            } else {
                s2.freq /= selected_interval;
            }
            music.insert(t, s2);
        } else {
            if s3.freq < 100. || (chance(seed * 123048132 + i * 3194719879, 0.5) && s3.freq < 1000.) {
                s3.freq *= selected_interval;
            } else {
                s3.freq /= selected_interval;
            }
            music.insert(t, s3);
        }
        t += 0.25;
    }

    music
}

use std::f32::consts::PI;

#[derive(Clone, Copy)]
pub struct Sound {
    pub freq: f32,
    pub A: f32,
    pub S: f32,
    pub R: f32,

    pub fmod_freq: f32,
    pub fmod_amt: f32,

    pub amplitude: f32,
    pub amp_lfo_freq: f32,
    pub amp_lfo_amount: f32,
}

impl Sound {
    pub fn new() -> Sound {
        Sound {
            freq: 440.0,
            A: 1.0,
            S: 1.0,
            R: 1.0,
            fmod_freq: 100.0,
            fmod_amt: 0.0,
            amplitude: 0.1,
            amp_lfo_freq: 20.0,
            amp_lfo_amount: 0.0, 
        }
    }

    // i like this, arbitrary mutation to immutable method chain. cya builder pattern
    pub fn but(&self, f: fn(&mut Sound)) -> Sound {
        let mut s = self.clone();
        f(&mut s);
        s
    }

    pub fn play(&self, sample_rate: f32) -> PlayingSound {
        PlayingSound {
            sample_rate,
            sample_count: 0,
            sound: self.clone(),
        }
    }
}

// seems liek its this but fmsynth works??
// is there still munted hf if i send nothing??

#[derive(Clone, Copy)]
pub struct PlayingSound {
    sample_rate: f32,
    sample_count: u32,
    sound: Sound,
}

impl PlayingSound {
    pub fn tick(&mut self) -> f32 {
        self.sample_count += 1; // overflows might be buggy

        let coeff = self.sample_count as f32 * 2.0 * PI / self.sample_rate;
            
        let fmod = (self.sound.fmod_freq * coeff).sin() * self.sound.fmod_amt + 1.0;
        let lfo = 1.0 - ((self.sound.amp_lfo_freq * coeff).sin() * self.sound.amp_lfo_amount);

        let attack_len = self.sound.A * self.sample_rate;
        let sustain_len = self.sound.S * self.sample_rate;
        let release_len = self.sound.R * self.sample_rate;

        let envelope = if self.sample_count as f32 <= attack_len {
            self.sample_count as f32 / attack_len
        } else if self.sample_count as f32 <= attack_len + sustain_len {
            1.0
        } else if self.sample_count as f32 <= attack_len + sustain_len + release_len {
            1.0 - ((self.sample_count as f32 - attack_len - sustain_len) / release_len)
        } else {
            0.0
        };

        self.sound.amplitude *
        envelope *
        lfo *
        (self.sound.freq * fmod * coeff).sin()
    }

    pub fn finished(&self) -> bool {
        let duration = self.sound.A + self.sound.S + self.sound.R;
        self.sample_count as f32 > duration * self.sample_rate
    }
}

// generic element type E
// keys are f32 times
// while let Some(elem) = tq.advance(t) {

// }

// tq.insert()

// this should solve some needs real good e.g. event system. which is basically what the sound player is.
// anything we want to happen at a specified time

use ordered_float::OrderedFloat;

pub struct TimeQueue<E> {
    elems: Vec<(OrderedFloat<f32>, E)>,
}

impl<E: Clone> TimeQueue<E> {
    fn upheap(&mut self, mut i: usize) {
        while i > 0 {
            if self.elems[i/2].0 > self.elems[i].0 {
                self.elems.swap(i, i/2);
                i /= 2;
            } else {
                return
            }
        }
    }

    fn downheap(&mut self, mut i: usize) {
        loop {
            // No children
            if i*2 + 1 >= self.elems.len() {
                return;
            }
            // no right child - just try left child
            if i*2 + 2 >= self.elems.len() {
                if self.elems[i].0 > self.elems[i*2+1].0 {
                    self.elems.swap(i, i*2+1);
                    i = i * 2 + 1;
                    continue;
                } else {
                    return;
                }
            }

            // both children
            let left_greatest = self.elems[i*2+1].0 >= self.elems[i*2+2].0;
            let greater_than_left = self.elems[i].0 > self.elems[i*2+1].0;
            let greater_than_right = self.elems[i].0 > self.elems[i*2+2].0;

            let swap_left = (greater_than_left && greater_than_right && !left_greatest) || (greater_than_left && !greater_than_right);
            
            if swap_left {
                self.elems.swap(i, i*2+1);
                i = i * 2 + 1;
                continue;
            }

            let swap_right = (greater_than_left && greater_than_right && left_greatest) || (greater_than_right && !greater_than_left);

            if swap_right {
                self.elems.swap(i, i*2+2);
                i = i * 2 + 2;
                continue;
            }

            return;
        }
    }

    pub fn new() -> TimeQueue<E> {
        TimeQueue {
            elems: Vec::new(),
        }
    }

    pub fn insert(&mut self, t: f32, elem: E) {
        self.elems.push((OrderedFloat(t), elem));
        self.upheap(self.elems.len() - 1);
    }

    pub fn advance(&mut self, t: f32) -> Option<E> {
        if self.elems.len() == 0 || OrderedFloat(t) < self.elems[0].0 {
            return None;
        }

        let ret = self.elems[0].1.clone();

        let last = self.elems.len() - 1;
        self.elems.swap(0, last);
        self.elems.truncate(self.elems.len() - 1);
        self.downheap(0);
        Some(ret)
    }
}