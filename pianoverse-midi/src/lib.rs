use std::sync::mpsc;

use nodi::{
    midly::Smf,
    midly::{MetaMessage, TrackEventKind},
    timers::{Ticker, TimeFormatError},
    Event, Sheet, Timer,
};

pub struct MidiPlayer<'a> {
    smf: Smf<'a>,
}

pub enum MidiEvent {
    Press(u32, u32),
    Release(u32),
    End,
}

impl<'a> MidiPlayer<'a> {
    pub fn load(bytes: &'a [u8]) -> Self {
        let smf = Smf::parse(bytes).unwrap();
        MidiPlayer { smf }
    }

    pub fn track_names(&self) -> Vec<String> {
        self.smf
            .tracks
            .iter()
            .map(|track| {
                track
                    .iter()
                    .find_map(|event| {
                        if let TrackEventKind::Meta(MetaMessage::TrackName(name)) = event.kind {
                            return Some(String::from_utf8_lossy(name).to_string());
                        }
                        None
                    })
                    .unwrap_or_else(|| "Unnamed".to_string())
            })
            .collect()
    }

    pub fn play(
        &self,
        track_idx: Vec<usize>,
        transpose: i8,
    ) -> Result<mpsc::Receiver<MidiEvent>, TimeFormatError> {
        let tracks: Vec<_> = track_idx
            .iter()
            .map(|&i| self.smf.tracks[i].clone())
            .collect();
        let sheet = Sheet::parallel(tracks.as_slice());

        let mut timer = Ticker::try_from(self.smf.header.timing)?;

        let (tx, rx) = mpsc::sync_channel(0);

        std::thread::spawn(move || {
            let mut counter = 0_u32;

            for mut moment in sheet {
                if !moment.is_empty() {
                    moment.transpose(transpose, false);

                    timer.sleep(counter);
                    counter = 0;

                    for event in &moment.events {
                        match event {
                            Event::Tempo(val) => timer.change_tempo(*val),
                            Event::Midi(msg) => match msg.message {
                                nodi::midly::MidiMessage::NoteOn { key, vel } => {
                                    tx.send(MidiEvent::Press(
                                        key.as_int() as u32,
                                        vel.as_int() as u32,
                                    ))
                                    .unwrap();
                                }
                                nodi::midly::MidiMessage::NoteOff { key, .. } => {
                                    tx.send(MidiEvent::Release(key.as_int() as u32)).unwrap();
                                }
                                _ => (),
                            },

                            _ => (),
                        };
                    }
                }

                counter += 1;
            }

            tx.send(MidiEvent::End).unwrap();
        });

        Ok(rx)
    }
}
