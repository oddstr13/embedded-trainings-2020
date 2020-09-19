#![deny(unused_must_use)]
#![no_main]
#![no_std]

use core::str;

use cortex_m_rt::entry;
use dk::ieee802154::{Channel, Error, Packet, TxPower};
use panic_log as _; // the panicking behavior

const TEN_MS: u32 = 10_000;

#[entry]
fn main() -> ! {
    // initializes the peripherals
    let board = dk::init().unwrap();
    let mut radio = board.radio;
    let mut timer = board.timer;

    // NOTE if you ran `change-channel` then you may need to update the channel here
    radio.set_channel(Channel::_20); // <- must match the Dongle's listening channel
    radio.set_txpower(TxPower::Pos8dBm);

    let mut packet = Packet::new();
    let msg = b"olleh";
    packet.copy_from_slice(msg);

    loop {
        log::info!(
            "sending: {}",
            str::from_utf8(msg).expect("message is not valid UTF-8")
        );
        radio.send(&packet);


        let res = radio.recv_timeout(&mut packet, &mut timer, TEN_MS);

        match res {
            Ok(crc) => {
                log::info!(
                    "received: {} (CRC={})",
                    str::from_utf8(&*packet).expect("response is not valid UTF-8"),
                    crc
                );
            }
            Err(Error::Crc(crc)) => log::error!("invalid CRC: {:06x}", crc),
            Err(Error::Timeout) => log::error!("no response within {} ms", TEN_MS / 1_000),
        }

        // TODO try uncommenting this line
        //timer.wait(Duration::from_secs_f32(5.0));
        dk::exit();
    }
    

}
