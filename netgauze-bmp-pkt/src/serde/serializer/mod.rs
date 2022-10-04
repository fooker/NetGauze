// Copyright (C) 2022-present The NetGauze Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Serializer library for BMP's wire protocol

use crate::{
    iana::{
        BMP_VERSION, PEER_FLAGS_IS_ADJ_RIB_OUT, PEER_FLAGS_IS_ASN2, PEER_FLAGS_IS_FILTERED,
        PEER_FLAGS_IS_IPV6, PEER_FLAGS_IS_POST_POLICY,
    },
    BmpMessage, BmpPeerType, InitiationInformation, InitiationMessage, PeerDownNotificationMessage,
    PeerDownNotificationReason, PeerHeader, PeerUpNotificationMessage, RouteMirroringMessage,
    RouteMirroringValue, RouteMonitoringMessage,
};
use byteorder::{NetworkEndian, WriteBytesExt};
use netgauze_bgp_pkt::{serde::serializer::BGPMessageWritingError, BGPMessage};
use netgauze_parse_utils::WritablePDU;
use netgauze_serde_macros::WritingError;
use std::{io::Write, net::IpAddr};

#[derive(WritingError, Eq, PartialEq, Clone, Debug)]
pub enum BmpMessageWritingError {
    StdIOError(#[from_std_io_error] String),
    RouteMonitoringMessageError(#[from] RouteMonitoringMessageWritingError),
    RouteMirroringMessageError(#[from] RouteMirroringMessageWritingError),
    InitiationMessageError(#[from] InitiationMessageWritingError),
    PeerUpNotificationMessageError(#[from] PeerUpNotificationMessageWritingError),
    PeerDownNotificationMessageError(#[from] PeerDownNotificationMessageWritingError),
}

impl WritablePDU<BmpMessageWritingError> for BmpMessage {
    const BASE_LENGTH: usize = 5;

    fn len(&self) -> usize {
        let len = match self {
            Self::RouteMonitoring(value) => value.len(),
            Self::StatisticsReport => todo!(),
            Self::PeerDownNotification(value) => value.len() + 1,
            Self::PeerUpNotification(value) => value.len(),
            Self::Initiation(value) => value.len() + 1,
            Self::Termination(_) => todo!(),
            Self::RouteMirroring(value) => value.len(),
            Self::Experimental251(value) => value.len(),
            Self::Experimental252(value) => value.len(),
            Self::Experimental253(value) => value.len(),
            Self::Experimental254(value) => value.len(),
        };
        Self::BASE_LENGTH + len
    }

    fn write<T: Write>(&self, writer: &mut T) -> Result<(), BmpMessageWritingError> {
        writer.write_u8(BMP_VERSION)?;
        writer.write_u32::<NetworkEndian>(self.len() as u32)?;
        writer.write_u8(self.get_type().into())?;
        match self {
            Self::RouteMonitoring(value) => {
                value.write(writer)?;
            }
            Self::StatisticsReport => {}
            Self::PeerDownNotification(value) => {
                value.write(writer)?;
            }
            Self::PeerUpNotification(value) => {
                value.write(writer)?;
            }
            Self::Initiation(value) => {
                value.write(writer)?;
            }
            Self::Termination(_) => {}
            Self::RouteMirroring(value) => {
                value.write(writer)?;
            }
            Self::Experimental251(value) => {
                writer.write_all(value)?;
            }
            Self::Experimental252(value) => {
                writer.write_all(value)?;
            }
            Self::Experimental253(value) => {
                writer.write_all(value)?;
            }
            Self::Experimental254(value) => {
                writer.write_all(value)?;
            }
        }
        Ok(())
    }
}

#[derive(WritingError, Eq, PartialEq, Clone, Debug)]
pub enum RouteMirroringMessageWritingError {
    StdIOError(#[from_std_io_error] String),
    PeerHeaderError(#[from] PeerHeaderWritingError),
}

impl WritablePDU<RouteMirroringMessageWritingError> for RouteMirroringMessage {
    const BASE_LENGTH: usize = 0;

    fn len(&self) -> usize {
        Self::BASE_LENGTH
            + self.peer_header.len()
            + self.mirrored().iter().map(|x| x.len()).sum::<usize>()
    }

    fn write<T: Write>(&self, _writer: &mut T) -> Result<(), RouteMirroringMessageWritingError> {
        todo!()
    }
}

#[inline]
const fn compute_peer_flags_value(
    ipv6: bool,
    post_policy: bool,
    asn2: bool,
    adj_rib_out: bool,
) -> u8 {
    let mut flags = 0;
    if ipv6 {
        flags |= PEER_FLAGS_IS_IPV6;
    }
    if post_policy {
        flags |= PEER_FLAGS_IS_POST_POLICY;
    }
    if asn2 {
        flags |= PEER_FLAGS_IS_ASN2
    }
    if adj_rib_out {
        flags |= PEER_FLAGS_IS_ADJ_RIB_OUT
    }
    flags
}

#[derive(WritingError, Eq, PartialEq, Clone, Debug)]
pub enum BmpPeerTypeWritingError {
    StdIOError(#[from_std_io_error] String),
}

impl WritablePDU<BmpPeerTypeWritingError> for BmpPeerType {
    /// 1-octet type and 1-octet flags
    const BASE_LENGTH: usize = 2;

    fn len(&self) -> usize {
        Self::BASE_LENGTH
    }

    fn write<T: Write>(&self, writer: &mut T) -> Result<(), BmpPeerTypeWritingError> {
        writer.write_u8(self.get_type().into())?;
        match self {
            Self::GlobalInstancePeer {
                ipv6,
                post_policy,
                asn2,
                adj_rib_out,
            }
            | Self::RdInstancePeer {
                ipv6,
                post_policy,
                asn2,
                adj_rib_out,
            }
            | Self::LocalInstancePeer {
                ipv6,
                post_policy,
                asn2,
                adj_rib_out,
            } => {
                let flags = compute_peer_flags_value(*ipv6, *post_policy, *asn2, *adj_rib_out);
                writer.write_u8(flags)?;
            }
            Self::LocRibInstancePeer { filtered } => {
                let flags = if *filtered { PEER_FLAGS_IS_FILTERED } else { 0 };
                writer.write_u8(flags)?;
            }
            Self::Experimental251 { flags }
            | Self::Experimental252 { flags }
            | Self::Experimental253 { flags }
            | Self::Experimental254 { flags } => {
                writer.write_u8(*flags)?;
            }
        }
        Ok(())
    }
}
#[derive(WritingError, Eq, PartialEq, Clone, Debug)]
pub enum PeerHeaderWritingError {
    StdIOError(#[from_std_io_error] String),
    BmpPeerTypeError(#[from] BmpPeerTypeWritingError),
}

impl WritablePDU<PeerHeaderWritingError> for PeerHeader {
    ///  1-octet peer type
    ///  1-octet peer flags
    ///  8-octets peer Distinguisher
    /// 16-octets peer address
    ///  4-octets peer AS
    ///  4-octets peer BGP ID
    ///  4-octets Timestamp (Seconds)
    ///  4-octets Timestamp (Microseconds)
    const BASE_LENGTH: usize = 42;

    fn len(&self) -> usize {
        Self::BASE_LENGTH
    }

    fn write<T: Write>(&self, writer: &mut T) -> Result<(), PeerHeaderWritingError> {
        self.peer_type.write(writer)?;
        match self.distinguisher() {
            None => writer.write_u64::<NetworkEndian>(0)?,
            Some(value) => writer.write_u64::<NetworkEndian>(*value)?,
        }
        match self.address() {
            Some(IpAddr::V4(ipv4)) => {
                writer.write_u64::<NetworkEndian>(0)?;
                writer.write_u32::<NetworkEndian>(0)?;
                writer.write_all(&ipv4.octets())?;
            }
            Some(IpAddr::V6(ipv6)) => {
                writer.write_all(&ipv6.octets())?;
            }
            None => writer.write_u128::<NetworkEndian>(0)?,
        }
        writer.write_u32::<NetworkEndian>(*self.peer_as())?;
        writer.write_all(&self.bgp_id().octets())?;
        match self.timestamp() {
            None => writer.write_u64::<NetworkEndian>(0)?,
            Some(time) => {
                writer.write_u32::<NetworkEndian>(time.timestamp() as u32)?;
                writer.write_u32::<NetworkEndian>(time.timestamp_subsec_micros())?;
            }
        }
        Ok(())
    }
}

#[derive(WritingError, Eq, PartialEq, Clone, Debug)]
pub enum RouteMirroringValueWritingError {
    StdIOError(#[from_std_io_error] String),
}

impl WritablePDU<RouteMirroringValueWritingError> for RouteMirroringValue {
    const BASE_LENGTH: usize = 0;

    fn len(&self) -> usize {
        todo!()
    }

    fn write<T: Write>(&self, _writer: &mut T) -> Result<(), RouteMirroringValueWritingError> {
        todo!()
    }
}

#[derive(WritingError, Eq, PartialEq, Clone, Debug)]
pub enum RouteMonitoringMessageWritingError {
    StdIOError(#[from_std_io_error] String),
    PeerHeaderError(#[from] PeerHeaderWritingError),
    BGPMessageError(#[from] BGPMessageWritingError),
}

impl WritablePDU<RouteMonitoringMessageWritingError> for RouteMonitoringMessage {
    const BASE_LENGTH: usize = 1;

    fn len(&self) -> usize {
        Self::BASE_LENGTH
            + self.peer_header.len()
            + self
                .updates()
                .iter()
                .map(|update| BGPMessage::Update(update.clone()).len())
                .sum::<usize>()
    }

    fn write<T: Write>(&self, writer: &mut T) -> Result<(), RouteMonitoringMessageWritingError> {
        self.peer_header.write(writer)?;
        for update in self.updates() {
            BGPMessage::Update(update.clone()).write(writer)?;
        }
        Ok(())
    }
}

#[derive(WritingError, Eq, PartialEq, Clone, Debug)]
pub enum InitiationMessageWritingError {
    StdIOError(#[from_std_io_error] String),
    InitiationInformationError(#[from] InitiationInformationWritingError),
}

impl WritablePDU<InitiationMessageWritingError> for InitiationMessage {
    const BASE_LENGTH: usize = 0;

    fn len(&self) -> usize {
        Self::BASE_LENGTH + self.information().iter().map(|x| x.len()).sum::<usize>()
    }

    fn write<T: Write>(&self, writer: &mut T) -> Result<(), InitiationMessageWritingError> {
        for info in self.information() {
            info.write(writer)?;
        }
        Ok(())
    }
}

#[derive(WritingError, Eq, PartialEq, Clone, Debug)]
pub enum InitiationInformationWritingError {
    StdIOError(#[from_std_io_error] String),
}

impl WritablePDU<InitiationInformationWritingError> for InitiationInformation {
    const BASE_LENGTH: usize = 4;

    fn len(&self) -> usize {
        Self::BASE_LENGTH
            + match self {
                Self::String(value) => value.len(),
                Self::SystemDescription(value) => value.len(),
                Self::SystemName(value) => value.len(),
                Self::VrfTableName(value) => value.len(),
                Self::AdminLabel(value) => value.len(),
                Self::Experimental65531(value) => value.len(),
                Self::Experimental65532(value) => value.len(),
                Self::Experimental65533(value) => value.len(),
                Self::Experimental65534(value) => value.len(),
            }
    }

    fn write<T: Write>(&self, writer: &mut T) -> Result<(), InitiationInformationWritingError> {
        writer.write_u16::<NetworkEndian>(self.get_type().into())?;
        match self {
            Self::String(value) => {
                let bytes = value.as_bytes();
                writer.write_u16::<NetworkEndian>(bytes.len() as u16)?;
                writer.write_all(bytes)?;
            }
            Self::SystemDescription(value) => {
                let bytes = value.as_bytes();
                writer.write_u16::<NetworkEndian>(bytes.len() as u16)?;
                writer.write_all(bytes)?;
            }
            Self::SystemName(value) => {
                let bytes = value.as_bytes();
                writer.write_u16::<NetworkEndian>(bytes.len() as u16)?;
                writer.write_all(bytes)?;
            }
            Self::VrfTableName(value) => {
                let bytes = value.as_bytes();
                writer.write_u16::<NetworkEndian>(bytes.len() as u16)?;
                writer.write_all(bytes)?;
            }
            Self::AdminLabel(value) => {
                let bytes = value.as_bytes();
                writer.write_u16::<NetworkEndian>(bytes.len() as u16)?;
                writer.write_all(bytes)?;
            }
            Self::Experimental65531(value) => {
                writer.write_u16::<NetworkEndian>(value.len() as u16)?;
                writer.write_all(value)?;
            }
            Self::Experimental65532(value) => {
                writer.write_u16::<NetworkEndian>(value.len() as u16)?;
                writer.write_all(value)?;
            }
            Self::Experimental65533(value) => {
                writer.write_u16::<NetworkEndian>(value.len() as u16)?;
                writer.write_all(value)?;
            }
            Self::Experimental65534(value) => {
                writer.write_u16::<NetworkEndian>(value.len() as u16)?;
                writer.write_all(value)?;
            }
        }
        Ok(())
    }
}

#[derive(WritingError, Eq, PartialEq, Clone, Debug)]
pub enum PeerUpNotificationMessageWritingError {
    StdIOError(#[from_std_io_error] String),
    PeerHeaderError(#[from] PeerHeaderWritingError),
    BGPMessageError(#[from] BGPMessageWritingError),
    InitiationInformationError(#[from] InitiationInformationWritingError),
}

impl WritablePDU<PeerUpNotificationMessageWritingError> for PeerUpNotificationMessage {
    // 16 local addr + 2 local port + 2 remote port
    const BASE_LENGTH: usize = 21;

    fn len(&self) -> usize {
        Self::BASE_LENGTH
            + self.peer_header.len()
            + self.sent_message.len()
            + self.received_message.len()
            + self.information().iter().map(|x| x.len()).sum::<usize>()
    }

    fn write<T: Write>(&self, writer: &mut T) -> Result<(), PeerUpNotificationMessageWritingError> {
        self.peer_header().write(writer)?;
        match self.local_address {
            IpAddr::V4(addr) => {
                writer.write_u64::<NetworkEndian>(0)?;
                writer.write_u32::<NetworkEndian>(0)?;
                writer.write_all(&addr.octets())?;
            }
            IpAddr::V6(addr) => writer.write_all(&addr.octets())?,
        }
        writer.write_u16::<NetworkEndian>(self.local_port.unwrap_or_default())?;
        writer.write_u16::<NetworkEndian>(self.remote_port.unwrap_or_default())?;

        self.sent_message().write(writer)?;
        self.received_message.write(writer)?;
        for info in &self.information {
            info.write(writer)?;
        }
        Ok(())
    }
}

#[derive(WritingError, Eq, PartialEq, Clone, Debug)]
pub enum PeerDownNotificationMessageWritingError {
    StdIOError(#[from_std_io_error] String),
    PeerHeaderError(#[from] PeerHeaderWritingError),
    InitiationInformationError(#[from] InitiationInformationWritingError),
    PeerDownNotificationReasonError(#[from] PeerDownNotificationReasonWritingError),
}

impl WritablePDU<PeerDownNotificationMessageWritingError> for PeerDownNotificationMessage {
    // 1 reason
    const BASE_LENGTH: usize = 0;

    fn len(&self) -> usize {
        Self::BASE_LENGTH + self.peer_header.len() + self.reason.len()
    }

    fn write<T: Write>(
        &self,
        writer: &mut T,
    ) -> Result<(), PeerDownNotificationMessageWritingError> {
        self.peer_header.write(writer)?;
        self.reason.write(writer)?;
        Ok(())
    }
}

#[derive(WritingError, Eq, PartialEq, Clone, Debug)]
pub enum PeerDownNotificationReasonWritingError {
    StdIOError(#[from_std_io_error] String),
    PeerHeaderError(#[from] PeerHeaderWritingError),
    BGPMessageError(#[from] BGPMessageWritingError),
    InitiationInformationError(#[from] InitiationInformationWritingError),
}

impl WritablePDU<PeerDownNotificationReasonWritingError> for PeerDownNotificationReason {
    // 1 reason
    const BASE_LENGTH: usize = 1;

    fn len(&self) -> usize {
        Self::BASE_LENGTH
            + match self {
                Self::LocalSystemClosedNotificationPduFollows(msg) => msg.len(),
                Self::LocalSystemClosedFsmEventFollows(_) => 2,
                Self::RemoteSystemClosedNotificationPduFollows(msg) => msg.len(),
                Self::RemoteSystemClosedNoData => 0,
                Self::PeerDeConfigured => 0,
                Self::LocalSystemClosedTlvDataFollows(info) => info.len(),
                Self::Experimental251(data) => data.len(),
                Self::Experimental252(data) => data.len(),
                Self::Experimental253(data) => data.len(),
                Self::Experimental254(data) => data.len(),
            }
    }

    fn write<T: Write>(
        &self,
        writer: &mut T,
    ) -> Result<(), PeerDownNotificationReasonWritingError> {
        writer.write_u8(self.get_type().into())?;
        match self {
            Self::LocalSystemClosedNotificationPduFollows(msg) => msg.write(writer)?,
            Self::LocalSystemClosedFsmEventFollows(value) => {
                writer.write_u16::<NetworkEndian>(*value)?
            }
            Self::RemoteSystemClosedNotificationPduFollows(msg) => msg.write(writer)?,
            Self::RemoteSystemClosedNoData => {}
            Self::PeerDeConfigured => {}
            Self::LocalSystemClosedTlvDataFollows(info) => info.write(writer)?,
            Self::Experimental251(data) => writer.write_all(&data[0..])?,
            Self::Experimental252(data) => writer.write_all(&data[0..])?,
            Self::Experimental253(data) => writer.write_all(&data[0..])?,
            Self::Experimental254(data) => writer.write_all(&data[0..])?,
        }
        Ok(())
    }
}
