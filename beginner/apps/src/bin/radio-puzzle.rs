#![deny(unused_must_use)]
#![no_main]
#![no_std]

use core::str;

use cortex_m_rt::entry;
use dk::ieee802154::{Channel, Packet};
use panic_log as _; // the panicking behavior

const TEN_MS: u32 = 10_000;

#[entry]
fn main() -> ! {
    let board = dk::init().unwrap();
    let mut radio = board.radio;
    let mut timer = board.timer;

    let mut reverse_key: [Option<u8>; 256] = [None; 256];
    let mut key: [Option<u8>; 256] = [None; 256];

    let mut seen = [false; 256];
    let mut challenge = [0 as u8; Packet::CAPACITY as usize];
    let mut challenge_len = 0;
    //log::info!("{:?}", key[32]);

    // puzzle.hex uses channel 25 by default
    // NOTE if you ran `change-channel` then you may need to update the channel here
    radio.set_channel(Channel::_25); // <- must match the Dongle's listening channel

    let mut packet = Packet::new();

    let msg = b"";
    packet.copy_from_slice(msg);
    log::info!("sending: {}", str::from_utf8(msg).expect("msg was not valid UTF-8 data"));
    radio.send(&packet);
    if radio.recv_timeout(&mut packet, &mut timer, TEN_MS).is_ok() {
        log::info!("received: {}", str::from_utf8(&packet).expect("response was not valid UTF-8 data"));
        for ch in packet.into_iter() {
            seen[*ch as usize] = true;
        }
        challenge_len = packet.len();
        
        for i in 0..challenge_len as usize {
            challenge[i] = packet[i];
        }
    } else {
        log::error!("no response or response packet was corrupted");
    }

    let mut abort;
    loop {
        for ch in 0..255 as usize {
            if seen[ch] && reverse_key[ch].is_none() {

                packet.copy_from_slice(&[ch as u8]);
                radio.send(&packet);
                if radio.recv_timeout(&mut packet, &mut timer, TEN_MS).is_ok() {
                    let out_ch = packet.get(0);
                    match out_ch {
                        None => (),
                        Some(v) => {
                            reverse_key[ch] = Some(*v);
                            seen[*v as usize] = true;
                            log::info!("{} -> {}", str::from_utf8(&[ch as u8]).unwrap_or("??"), str::from_utf8(&[*v]).unwrap_or("??"));
                        },
                    }
                }
            }
        }


        abort = true;
        for ch in 0..255 as usize {
            if seen[ch] && reverse_key[ch].is_none() {
                abort = false;
                break;
            }
        }
        if abort {
            break;
        }
    }


    for i in 0..255 as u8 {
        let ch = reverse_key[i as usize].unwrap_or(0);
        if ch != 0 {
            key[ch as usize] = Some(i);
        }
    }

    log::info!("challenge: {}", str::from_utf8(&challenge[..challenge_len as usize]).expect("response was not valid UTF-8 data"));

    for i in 0..challenge_len as usize {
        challenge[i] = key[challenge[i] as usize].unwrap_or(b' ');
    }

    log::info!("decoded: {}", str::from_utf8(&challenge[..challenge_len as usize]).expect("response was not valid UTF-8 data"));

    packet.copy_from_slice(&challenge[..challenge_len as usize]);
    log::info!("sending: {}", str::from_utf8(&packet).expect("msg was not valid UTF-8 data"));
    radio.send(&packet);
    if radio.recv_timeout(&mut packet, &mut timer, TEN_MS).is_ok() {
        log::info!("received: {}", str::from_utf8(&packet).expect("response was not valid UTF-8 data"));
    } else {
        log::error!("no response or response packet was corrupted");
    }


    dk::exit()
}
