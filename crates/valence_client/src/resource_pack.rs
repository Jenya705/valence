use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use valence_core::text::Text;
use valence_packet::packets::play::{ResourcePackSendS2c, ResourcePackStatusC2s};

use super::*;
use crate::event_loop::{EventLoopPreUpdate, PacketEvent};

pub(super) fn build(app: &mut App) {
    app.add_event::<ResourcePackStatusEvent>()
        .add_systems(EventLoopPreUpdate, handle_resource_pack_status);
}

#[derive(Event, Copy, Clone, PartialEq, Eq, Debug)]
pub struct ResourcePackStatusEvent {
    pub client: Entity,
    pub status: ResourcePackStatusC2s,
}

impl Client {
    /// Requests that the client download and enable a resource pack.
    ///
    /// # Arguments
    /// * `url` - The URL of the resource pack file.
    /// * `hash` - The SHA-1 hash of the resource pack file. Any value other
    ///   than a 40-character hexadecimal string is ignored by the client.
    /// * `forced` - Whether a client should be kicked from the server upon
    ///   declining the pack (this is enforced client-side)
    /// * `prompt_message` - A message to be displayed with the resource pack
    ///   dialog.
    pub fn set_resource_pack(
        &mut self,
        url: &str,
        hash: &str,
        forced: bool,
        prompt_message: Option<Text>,
    ) {
        self.write_packet(&ResourcePackSendS2c {
            url,
            hash,
            forced,
            prompt_message: prompt_message.map(|t| t.into()),
        });
    }
}

fn handle_resource_pack_status(
    mut packets: EventReader<PacketEvent>,
    mut events: EventWriter<ResourcePackStatusEvent>,
) {
    for packet in packets.iter() {
        if let Some(pkt) = packet.decode::<ResourcePackStatusC2s>() {
            events.send(ResourcePackStatusEvent {
                client: packet.client,
                status: pkt,
            })
        }
    }
}
