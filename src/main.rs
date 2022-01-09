use binrw::BinRead;
use std::{error::Error, io::Cursor, net::UdpSocket};

use clap::Parser;
use f1_2021_rust::{Body, CarDamage, Packet};

#[derive(Debug, Parser)]
struct Params {
    /// The webhook to send stuff to
    webhook: String,

    /// The hostname the game is sending to
    #[clap(short, long, default_value = "127.0.0.1")]
    host: String,

    /// The port the game is sending to
    #[clap(short, long, default_value_t = 20777u16)]
    port: u16,
}

fn alert(damage: &CarDamage, path: &str) -> Result<(), Box<dyn Error>> {
    ureq::post(path).send_json(ureq::json!({
        "username": "damage tracker",
        "embeds": [
            {
                "title": "DAMAGE TAKEN",
                "type": "rich",
                "fields": [
                    {
                        "name": "Tyre Damage",
                        "value": format!("{:?}", damage.tyre_damage),
                    },
                    {
                        "name": "Brake Damage",
                        "value": format!("{:?}", damage.brake_damage),
                    },
                    {
                        "name": "Front Left Wing Damage",
                        "value": format!("{:?}", damage.front_left_wing_damage),
                    },
                    {
                        "name": "Front Right Wing Damage",
                        "value": format!("{:?}", damage.front_right_wing_damage),
                    },
                    {
                        "name": "Rear Wing Damage",
                        "value": format!("{:?}", damage.rear_wing_damage),
                    },
                    {
                        "name": "Floor Damage",
                        "value": format!("{:?}", damage.floor_wing_damage),
                    },
                    {
                        "name": "Diffuser Damage",
                        "value": format!("{:?}", damage.diffuser_damage),
                    },
                ]
            }
        ]
    }))?;

    Ok(())
}

fn compare_fields_we_care_about(lhs: &CarDamage, rhs: &CarDamage) -> bool {
    let lhs_ = (
        lhs.tyre_damage.clone(),
        lhs.brake_damage.clone(),
        lhs.front_left_wing_damage,
        lhs.front_right_wing_damage,
        lhs.rear_wing_damage,
        lhs.floor_wing_damage,
        lhs.diffuser_damage,
        lhs.sidepod_damage,
        lhs.gearbox_damage,
        lhs.engine_damage,
    );

    let rhs_ = (
        rhs.tyre_damage.clone(),
        rhs.brake_damage.clone(),
        rhs.front_left_wing_damage,
        rhs.front_right_wing_damage,
        rhs.rear_wing_damage,
        rhs.floor_wing_damage,
        rhs.diffuser_damage,
        rhs.sidepod_damage,
        rhs.gearbox_damage,
        rhs.engine_damage,
    );

    lhs_ == rhs_
}

fn main() -> Result<(), Box<dyn Error>> {
    let params = Params::parse();

    println!("Sure thing!");

    let socket = UdpSocket::bind(("0.0.0.0", 0))?;
    socket.connect((params.host.as_str(), params.port))?;

    let mut buf = [0; 1500];

    let mut seen_car_damage: Option<CarDamage> = None;

    loop {
        if let Ok(_) = socket.recv(&mut buf) {
            let msg: Packet = match Packet::read(&mut Cursor::new(&buf)) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("{:?}", e);
                    continue;
                }
            };

            let player_car_idx = msg.header.player_car_idx;

            if let Body::CarDamage(damages) = msg.body {
                let damage = &damages.car_damages[player_car_idx as usize];

                if let Some(known_damage) = seen_car_damage.as_ref() {
                    if !compare_fields_we_care_about(known_damage, damage) {
                        alert(damage, params.webhook.as_str())?;
                    }
                }

                seen_car_damage = Some(damage.clone());
            }
        }
    }
}
