use std::io::{self, ErrorKind};
use std::net::SocketAddr;

use anyhow::bail;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use uuid::Uuid;
use valence_core::protocol::var_int::VarInt;
use valence_core::PROTOCOL_VERSION;
use valence_packet::packets::handshaking::handshake_c2s::HandshakeNextState;
use valence_packet::packets::handshaking::HandshakeC2s;
use valence_packet::packets::login::{
    LoginCompressionS2c, LoginHelloC2s, LoginHelloS2c, LoginSuccessS2c,
};
use valence_packet::packets::play::{
    KeepAliveC2s, KeepAliveS2c, PlayerPositionLookS2c, PositionAndOnGroundC2s, TeleportConfirmC2s,
};
use valence_packet::protocol::decode::PacketDecoder;
use valence_packet::protocol::encode::PacketEncoder;
use valence_packet::protocol::Packet;

pub struct SessionParams<'a> {
    pub socket_addr: SocketAddr,
    pub session_name: &'a str,
    pub read_buffer_size: usize,
}

pub async fn make_session<'a>(params: &SessionParams<'a>) -> anyhow::Result<()> {
    let sock_addr = params.socket_addr;
    let sess_name = params.session_name;
    let rb_size = params.read_buffer_size;

    let mut conn = match TcpStream::connect(sock_addr).await {
        Ok(conn) => {
            println!("{sess_name} connected");
            conn
        }
        Err(err) => {
            eprintln!("{sess_name} connection failed");
            return Err(err.into());
        }
    };

    conn.set_nodelay(true)?;

    let mut dec = PacketDecoder::new();
    let mut enc = PacketEncoder::new();

    let server_addr_str = sock_addr.ip().to_string();

    let handshake_pkt = HandshakeC2s {
        protocol_version: VarInt(PROTOCOL_VERSION),
        server_address: &server_addr_str,
        server_port: sock_addr.port(),
        next_state: HandshakeNextState::Login,
    };

    enc.append_packet(&handshake_pkt)?;

    enc.append_packet(&LoginHelloC2s {
        username: sess_name,
        profile_id: Some(Uuid::new_v4()),
    })?;

    let write_buf = enc.take();
    conn.write_all(&write_buf).await?;

    loop {
        dec.reserve(rb_size);

        let mut read_buf = dec.take_capacity();

        conn.readable().await?;

        match conn.try_read_buf(&mut read_buf) {
            Ok(0) => return Err(io::Error::from(ErrorKind::UnexpectedEof).into()),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e.into()),
            Ok(_) => (),
        };

        dec.queue_bytes(read_buf);

        if let Ok(Some(frame)) = dec.try_next_packet() {
            match frame.id {
                LoginCompressionS2c::ID => {
                    let packet: LoginCompressionS2c = frame.decode()?;
                    let threshold = packet.threshold.0 as u32;

                    dec.set_compression(Some(threshold));
                    enc.set_compression(Some(threshold));
                }

                LoginSuccessS2c::ID => {
                    break;
                }

                LoginHelloS2c::ID => {
                    bail!("encryption not implemented");
                }

                _ => (),
            }
        }
    }

    println!("{sess_name} logged in");

    loop {
        while let Some(frame) = dec.try_next_packet()? {
            match frame.id {
                KeepAliveS2c::ID => {
                    let packet: KeepAliveS2c = frame.decode()?;
                    enc.clear();

                    enc.append_packet(&KeepAliveC2s { id: packet.id })?;
                    conn.write_all(&enc.take()).await?;
                }

                PlayerPositionLookS2c::ID => {
                    let packet: PlayerPositionLookS2c = frame.decode()?;
                    enc.clear();

                    enc.append_packet(&TeleportConfirmC2s {
                        teleport_id: packet.teleport_id,
                    })?;

                    enc.append_packet(&PositionAndOnGroundC2s {
                        position: packet.position,
                        on_ground: true,
                    })?;

                    conn.write_all(&enc.take()).await?;
                }
                _ => (),
            }
        }

        dec.reserve(rb_size);

        let mut read_buf = dec.take_capacity();

        conn.readable().await?;

        match conn.try_read_buf(&mut read_buf) {
            Ok(0) => return Err(io::Error::from(ErrorKind::UnexpectedEof).into()),
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e.into()),
            Ok(_) => (),
        };

        dec.queue_bytes(read_buf);
    }
}
