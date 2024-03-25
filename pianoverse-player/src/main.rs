use std::sync::Arc;

use anyhow::Context;
use clap::{arg, Command};
use inquire::list_option::ListOption;

use pianoverse_midi::{MidiEvent, MidiPlayer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let c = Command::new("midiverse")
        .about("MIDI files player for pianoverse.net")
        .author("altfoxie")
        .args(&[
            arg!(-r --room <ROOM> "Room name to join/create").default_value("midiverse"),
            arg!(-p --private "Create a private room"),
            arg!(-t --transpose <N> "Transpose MIDI notes for N semitones")
                .allow_negative_numbers(true),
            arg!([input] "MIDI file to play").required(true),
        ])
        .get_matches();

    let room = c.get_one::<String>("room").unwrap();
    let private = c.get_one::<bool>("private").unwrap();
    let input = c.get_one::<String>("input").unwrap();
    let transpose = c
        .get_one::<String>("transpose")
        .and_then(|s| s.parse::<i8>().ok())
        .unwrap_or(0);

    let data = std::fs::read(input).context("Failed to read MIDI file")?;
    let player = MidiPlayer::load(data.as_slice()).context("Failed to load MIDI file")?;
    let track_idx: Vec<usize> = inquire::MultiSelect::new(
        "Select tracks to play",
        player
            .track_names()
            .iter()
            .enumerate()
            .map(|(i, name)| ListOption::new(i, name.clone()))
            .collect(),
    )
    .with_all_selected_by_default()
    .prompt()?
    .iter()
    .map(|i| i.index)
    .collect();

    let client = Arc::new(pianoverse_client::Client::connect().await.unwrap());
    {
        let client = client.clone();
        tokio::spawn(async move {
            loop {
                let _msg = client.recv().await.unwrap();
            }
        });
    }

    client
        .join_or_create_room(room, *private)
        .await
        .context("Failed to join/create room")?;

    let rx = player
        .play(track_idx, transpose)
        .context("Failed to start playing MIDI")?;

    for event in rx {
        match event {
            MidiEvent::Press(key, vel) => {
                println!("Press\tKey: {}, Velocity: {}", key, vel);
                client.press(key, vel).await.unwrap();
            }
            MidiEvent::Release(key) => {
                println!("Release\tKey: {}", key);
                client.release(key).await.unwrap();
            }
            MidiEvent::End => {
                break;
            }
        }
    }

    Ok(())
}
