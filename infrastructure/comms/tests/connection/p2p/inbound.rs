//  Copyright 2019 The Tari Project
//
//  Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
//  following conditions are met:
//
//  1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
//  disclaimer.
//
//  2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
//  following disclaimer in the documentation and/or other materials provided with the distribution.
//
//  3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
//  products derived from this software without specific prior written permission.
//
//  THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
//  INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
//  DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
//  SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
//  SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
//  WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
//  USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use crate::support::{self, utils as support_utils};
use std::time::Duration;
use tari_comms::connection::{
    p2p::inbound::InboundConnection,
    zmq::{curve_keypair, Context, InprocAddress},
    ConnectionError,
};

#[test]
fn receive_timeout() {
    let ctx = Context::new();

    let addr = InprocAddress::random();

    let conn = InboundConnection::new(&ctx).bind(&addr).unwrap();

    let result = conn.receive(1);
    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        ConnectionError::Timeout => {},
        _ => panic!("Unexpected error type: {:?}", err),
    }
}

#[test]
fn receive_inproc() {
    let ctx = Context::new();

    let addr = InprocAddress::random();

    let req_rep_pattern = support::comms_patterns::async_request_reply();

    let conn = InboundConnection::new(&ctx).bind(&addr).unwrap();

    let signal = req_rep_pattern
        .set_endpoint(addr.clone())
        .set_identity("boba")
        .set_send_data(vec![
            "Just".as_bytes().to_vec(),
            "Three".as_bytes().to_vec(),
            "Messages".as_bytes().to_vec(),
        ])
        .run(ctx.clone());

    let frames = conn.receive(1000).unwrap();
    assert_eq!(frames.len(), 4);
    assert_eq!("boba".as_bytes(), frames[0].as_slice());
    assert_eq!("Just".as_bytes(), frames[1].as_slice());
    assert_eq!("Three".as_bytes(), frames[2].as_slice());
    assert_eq!("Messages".as_bytes(), frames[3].as_slice());

    conn.send(&["boba", "OK"]).unwrap();

    // Wait for pattern to exit
    signal.recv_timeout(Duration::from_millis(200)).unwrap();
}

#[test]
fn receive_encrypted_tcp() {
    let ctx = Context::new();

    let addr = support_utils::find_available_tcp_net_address("127.0.0.1").unwrap();

    let req_rep_pattern = support::comms_patterns::async_request_reply();

    let (sk, pk) = curve_keypair::generate().unwrap();

    let conn = InboundConnection::new(&ctx)
        .set_curve_secret_key(sk)
        .bind(&addr)
        .unwrap();

    let signal = req_rep_pattern
        .set_endpoint(addr.clone())
        .set_identity("the dude")
        .set_public_key(pk)
        .set_send_data(vec![(0..255).map(|i| i as u8).collect::<Vec<_>>()])
        .run(ctx.clone());

    let frames = conn.receive(1000).unwrap();
    assert_eq!(frames.len(), 2);

    conn.send(&["the dude", "OK"]).unwrap();

    // Wait for pattern to exit
    signal.recv_timeout(Duration::from_millis(200)).unwrap();
}