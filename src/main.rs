// extern crate gnome;
use animaterm::prelude::*;
use async_std::task::sleep;
use async_std::task::spawn;
// use core::panic;
use gnome::prelude::*;
use std::env::args;
// use std::fs;
// use std::net::IpAddr;
// use std::net::Ipv4Addr;
// use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

#[async_std::main]
async fn main() {
    let dir: String = args().nth(1).unwrap().parse().unwrap();
    let capture_keyboard = true;
    let cols = Some(40);
    let rows = None; // use all rows available
    let glyph = Some(Glyph::default()); // initially fill the screen with this
                                        // You can crank refresh_timeout down, but anything below 1ms won't make a difference,
                                        // other than high CPU usage.
                                        // With default 30ms you get as high as 33 FPS, probably enough for a terminal application.
    let refresh_timeout = Some(Duration::from_millis(10));
    let mut mgr = Manager::new(capture_keyboard, cols, rows, glyph, refresh_timeout);

    // let network_settings = NetworkSettings {
    //     pub_ip: IpAddr::V4(Ipv4Addr::from([46, 232, 214, 235])),
    //     pub_port: 14320,
    //     nat_type: Nat::SymmetricWithPortControl,
    //     port_allocation: (PortAllocationRule::FullCone, 0),
    // };
    // let neighbor_network_settings = NetworkSettings {
    //     pub_ip: IpAddr::V4(Ipv4Addr::from([46, 112, 68, 96])),
    //     pub_port: 19859,
    //     nat_type: Nat::Symmetric,
    //     port_allocation: (PortAllocationRule::FullCone, 0),
    // };
    // let network_settings = NetworkSettings {
    //     pub_ip: IpAddr::V4(Ipv4Addr::from([100, 116, 51, 23])),
    //     pub_port: 1030,
    //     nat_type: Nat::SymmetricWithPortControl,
    //     port_allocation: (PortAllocationRule::FullCone, 0),
    // };
    // let neighbor_network_settings = NetworkSettings {
    //     pub_ip: IpAddr::V4(Ipv4Addr::from([100, 106, 116, 115])),
    //     pub_port: 1030,
    //     nat_type: Nat::SymmetricWithPortControl,
    //     port_allocation: (PortAllocationRule::FullCone, 0),
    // };

    let (gnome_send, gnome_recv) = init(dir);
    let mut next_val = 1;
    let man_resp_result = gnome_recv.recv();
    let service_request;
    if let Ok(ManagerResponse::SwarmJoined(s_id, s_name, service_req, service_resp)) =
        man_resp_result
    {
        service_request = service_req;
        spawn(serve_user_responses(
            Duration::from_millis(30),
            service_resp,
        ));
    } else {
        return;
    }

    // TODO
    loop {
        // println!("loop start");
        if let Some(key) = mgr.read_key() {
            // println!("some key");
            match key {
                Key::J => {
                    let _ = gnome_send.send(ManagerRequest::JoinSwarm("trzat".to_string()));
                }
                Key::Q | Key::ShiftQ => {
                    let _ = gnome_send.send(ManagerRequest::Disconnect);
                    // keep_running = false;
                    break;
                }
                // TODO: this should be served separately by sending to user_req
                Key::B => {
                    let _ = service_request.send(Request::StartBroadcast);
                }
                Key::N => {
                    let _ = service_request.send(Request::ListNeighbors);
                }
                Key::S => {
                    let _ =
                        service_request.send(Request::AddData(Data::new(vec![next_val]).unwrap()));
                    next_val += 1;
                }
                Key::ShiftS => {
                    let data = vec![next_val; 1024];

                    let _ = service_request.send(Request::AddData(Data::new(data).unwrap()));
                    next_val += 1;
                }
                Key::ShiftU => {
                    let res =
                        service_request.send(Request::StartUnicast(GnomeId(15561580566906229863)));
                    println!("UnicastReq: {:?}", res);
                    // next_val += 1;
                }
                _ => println!(),
            }
        }
        if let Ok(gnome_response) = gnome_recv.try_recv() {
            match gnome_response {
                ManagerResponse::SwarmJoined(swarm_id, swarm_name, user_req, user_res) => {
                    // TODO: serve user_req
                    let sleep_time = Duration::from_millis(30);
                    spawn(serve_user_responses(sleep_time, user_res));
                }
            }
        }
        // sleep(Duration::from_millis(100)).await;
        // println!("loop end");
    }
    mgr.terminate();
}

async fn serve_user_responses(sleep_time: Duration, user_res: Receiver<Response>) {
    loop {
        let data = user_res.try_recv();
        if let Ok(resp) = data {
            match resp {
                Response::Broadcast(_s_id, c_id, recv_d) => {
                    spawn(serve_broadcast(c_id, Duration::from_millis(100), recv_d));
                }
                Response::Unicast(_s_id, c_id, recv_d) => {
                    spawn(serve_unicast(c_id, Duration::from_millis(100), recv_d));
                }
                Response::BroadcastOrigin(_s_id, c_id, send_d) => {
                    spawn(serve_broadcast_origin(
                        c_id,
                        Duration::from_millis(200),
                        send_d,
                    ));
                }
                Response::UnicastOrigin(_s_id, c_id, send_d) => {
                    spawn(serve_unicast_origin(
                        c_id,
                        Duration::from_millis(500),
                        send_d,
                    ));
                }
                _ => {
                    println!("Data received: {:?}", resp);
                }
            }
        } else {
            // println!("{:?}", data);
        }
        sleep(sleep_time).await;
    }
}
async fn serve_unicast(c_id: CastID, sleep_time: Duration, user_res: Receiver<Data>) {
    println!("Serving unicast {:?}", c_id);
    loop {
        let recv_res = user_res.try_recv();
        if let Ok(data) = recv_res {
            println!("U{:?}: {}", c_id, data);
        }
        sleep(sleep_time).await;
    }
}
async fn serve_broadcast(c_id: CastID, sleep_time: Duration, user_res: Receiver<Data>) {
    println!("Serving broadcast {:?}", c_id);
    loop {
        let recv_res = user_res.try_recv();
        if let Ok(data) = recv_res {
            println!("B{:?}: {}", c_id, data);
        }
        sleep(sleep_time).await;
    }
}
async fn serve_unicast_origin(c_id: CastID, sleep_time: Duration, user_res: Sender<Data>) {
    println!("Originating unicast {:?}", c_id);
    let mut i: u8 = 0;
    loop {
        let send_res = user_res.send(Data::new(vec![i]).unwrap());
        if send_res.is_ok() {
            println!("Unicasted {}", i);
        } else {
            println!(
                "Error while trying to unicast: {:?}",
                send_res.err().unwrap()
            );
        }
        i = i.wrapping_add(1);

        sleep(sleep_time).await;
    }
}
async fn serve_broadcast_origin(c_id: CastID, sleep_time: Duration, user_res: Sender<Data>) {
    println!("Originating broadcast {:?}", c_id);
    let mut i: u8 = 0;
    loop {
        let send_res = user_res.send(Data::new(vec![i]).unwrap());
        if send_res.is_ok() {
            println!("Broadcasted {}", i);
        } else {
            println!(
                "Error while trying to broadcast: {:?}",
                send_res.err().unwrap()
            );
        }
        i = i.wrapping_add(1);

        sleep(sleep_time).await;
    }
}
