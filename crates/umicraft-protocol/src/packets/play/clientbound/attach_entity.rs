// SPDX-License-Identifier: MIT OR Apache-2.0

//! `Attach Entity` (0x1B, clientbound, play): attaches an entity to another
//! (mounting or leashing).
//!
//! Layout (MCP `S1BPacketAttachEntity`, `readPacketData`):
//!
//! | field | width |
//! |---|---|
//! | entity id | i32 |
//! | vehicle entity id | i32 |
//! | leash | u8 (0 = riding, 1 = leash) |

use std::io::{Read, Write};

use crate::error::ProtocolError;
use crate::packets::codec::read_i32;
use crate::packets::EncodablePacket;

/// `Attach Entity` (0x1B, clientbound, play).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachEntity {
    /// Leash flag (0 = riding / mount, 1 = leash).
    pub leash: u8,
    /// The attached entity ID.
    pub entity_id: i32,
    /// The vehicle or leash holder entity ID (-1 if detached).
    pub vehicle_id: i32,
}

impl AttachEntity {
    /// Wire packet id.
    pub const ID: u8 = 0x1B;

    /// Reads the packet body.
    ///
    /// # Errors
    /// [`ProtocolError`] on truncation.
    pub fn decode(reader: &mut impl Read) -> Result<Self, ProtocolError> {
        let entity_id = read_i32(reader)?;
        let vehicle_id = read_i32(reader)?;
        let mut leash_buf = [0u8; 1];
        reader.read_exact(&mut leash_buf)?;
        let leash = leash_buf[0];
        Ok(Self {
            leash,
            entity_id,
            vehicle_id,
        })
    }
}

impl EncodablePacket for AttachEntity {
    fn packet_id(&self) -> u8 {
        Self::ID
    }

    fn encode_body(&self, writer: &mut impl Write) -> Result<(), ProtocolError> {
        writer.write_all(&self.entity_id.to_be_bytes())?;
        writer.write_all(&self.vehicle_id.to_be_bytes())?;
        writer.write_all(&[self.leash])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attach_entity_roundtrip() {
        let packet = AttachEntity {
            leash: 0,
            entity_id: 10,
            vehicle_id: 20,
        };

        let mut buf = Vec::new();
        packet.encode_body(&mut buf).unwrap();
        let decoded = AttachEntity::decode(&mut buf.as_slice()).unwrap();
        assert_eq!(packet, decoded);
    }

    /// The wire order of `S1BPacketEntityAttach`: the entity, the vehicle, then the
    /// leash flag. A decoder that read the flag first made every rider and every
    /// leash unrecognisable.
    #[test]
    fn the_wire_order_is_entity_vehicle_then_leash() {
        let bytes = [0, 0, 0, 7, 0, 0, 1, 2, 0];
        let packet = AttachEntity::decode(&mut bytes.as_slice()).unwrap();
        assert_eq!(
            packet,
            AttachEntity {
                leash: 0,
                entity_id: 7,
                vehicle_id: 258,
            }
        );
        let mut written = Vec::new();
        packet.encode_body(&mut written).unwrap();
        assert_eq!(written, bytes);
    }
}
