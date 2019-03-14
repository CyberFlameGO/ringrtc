//
// Copyright (C) 2019 Signal Messenger, LLC.
// All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only
//

//! WebRTC Peer Connection Interface
use std::ffi::CString;
use std::fmt;
use std::os::raw::c_char;

use crate::common::Result;
use crate::error::RingRtcError;
use crate::webrtc::data_channel::{
    RffiDataChannelInit,
    DataChannel,
};
use crate::webrtc::ice_candidate::IceCandidate;
use crate::webrtc::sdp_observer::{
    CreateSessionDescriptionObserver,
    SetSessionDescriptionObserver,
    SessionDescriptionInterface,
    RffiCreateSessionDescriptionObserver,
    RffiSessionDescriptionInterface,
    RffiSetSessionDescriptionObserver,
};

/// Incomplete type for C++ PeerConnectionInterface.
#[repr(C)]
pub struct RffiPeerConnectionInterface { _private: [u8; 0] }

/// Incomplete type for C++ DataChannelInterface.
#[repr(C)]
pub struct RffiDataChannelInterface { _private: [u8; 0] }

/// Rust wrapper around WebRTC C++ PeerConnectionInterface object.
pub struct PeerConnection
{
    /// Pointer to C++ PeerConnectionInterface.
    rffi_pc_interface: *const RffiPeerConnectionInterface,
}

impl fmt::Display for PeerConnection
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "pc_interface: {:p}", self.rffi_pc_interface)
    }
}

impl fmt::Debug for PeerConnection
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl PeerConnection
{
    /// Create a new Rust PeerConnection object from a WebRTC C++
    /// PeerConnectionInterface object.
    pub fn new(rffi_pc_interface: *const RffiPeerConnectionInterface) -> Self {
        Self {
            rffi_pc_interface,
        }
    }

    /// Rust wrapper around C++ PeerConnectionInterface::CreateDataChannel().
    pub fn create_data_channel(&self, label: String) -> Result<DataChannel> {
        let data_channel_label = CString::new(label)?;
        let data_channel_config = RffiDataChannelInit::new(true)?;

        let rffi_data_channel = unsafe {
            Rust_createDataChannel(self.rffi_pc_interface,
                                   data_channel_label.as_ptr(),
                                   &data_channel_config)
        };
        if rffi_data_channel.is_null() {
            return Err(RingRtcError::CreateDataChannel(data_channel_label.into_string()?).into());
        }

        let data_channel = DataChannel::new(rffi_data_channel);

        Ok(data_channel)
    }

    /// Rust wrapper around C++ webrtc::CreateSessionDescription(kOffer).
    pub fn create_offer(&self, csd_observer: &CreateSessionDescriptionObserver) {
        unsafe { Rust_createOffer(self.rffi_pc_interface, csd_observer.get_rffi_observer()) }
    }

    /// Rust wrapper around C++ PeerConnectionInterface::SetLocalDescription().
    pub fn set_local_description(&self,
                                 ssd_observer: &SetSessionDescriptionObserver,
                                 desc: &SessionDescriptionInterface) {
        unsafe { Rust_setLocalDescription(self.rffi_pc_interface,
                                          ssd_observer.get_rffi_observer(),
                                          desc.get_rffi_interface()) }
    }

    /// Rust wrapper around C++ webrtc::CreateSessionDescription(kAnswer).
    pub fn create_answer(&self, csd_observer: &CreateSessionDescriptionObserver) {
        unsafe { Rust_createAnswer(self.rffi_pc_interface, csd_observer.get_rffi_observer()) };
    }

    /// Rust wrapper around C++ PeerConnectionInterface::SetRemoteDescription().
    pub fn set_remote_description(&self,
                                  ssd_observer: &SetSessionDescriptionObserver,
                                  desc: &SessionDescriptionInterface) {
        unsafe { Rust_setRemoteDescription(self.rffi_pc_interface,
                                           ssd_observer.get_rffi_observer(),
                                           desc.get_rffi_interface()) };
    }

    /// Rust wrapper around C++ PeerConnectionInterface::AddIceCandidate().
    pub fn add_ice_candidate(&self, candidate: &IceCandidate) -> Result<()> {
        let clone = candidate.clone();
        let sdp_mid = CString::new(clone.sdp_mid)?;
        let sdp = CString::new(clone.sdp)?;
        let add_ok = unsafe {
            Rust_addIceCandidate(self.rffi_pc_interface,
                                 sdp_mid.as_ptr(), clone.sdp_mline_index, sdp.as_ptr())
        };
        if add_ok {
            Ok(())
        } else {
            Err(RingRtcError::AddIceCandidate.into())
        }
    }

}

extern {
    fn Rust_createOffer(pc_interface: *const RffiPeerConnectionInterface,
                        csd_observer: *const RffiCreateSessionDescriptionObserver);

    fn Rust_setLocalDescription(pc_interface: *const RffiPeerConnectionInterface,
                                ssd_observer: *const RffiSetSessionDescriptionObserver,
                                desc: *const RffiSessionDescriptionInterface);

    fn Rust_createAnswer(pc_interface: *const RffiPeerConnectionInterface,
                         csd_observer: *const RffiCreateSessionDescriptionObserver);

    fn Rust_setRemoteDescription(pc_interface: *const RffiPeerConnectionInterface,
                                 ssd_observer: *const RffiSetSessionDescriptionObserver,
                                 desc:         *const RffiSessionDescriptionInterface);

    fn Rust_createDataChannel(pc_interface: *const RffiPeerConnectionInterface,
                              label:        *const c_char,
                              config:       *const RffiDataChannelInit)
                              -> *const RffiDataChannelInterface;

    fn Rust_addIceCandidate(pc_interface:    *const RffiPeerConnectionInterface,
                            sdp_mid:         *const c_char,
                            sdp_mline_index: i32,
                            sdp:             *const c_char) -> bool;
}
