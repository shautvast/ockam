use ockam_core::{Encodable, Result, TransportMessage};
use ockam_transport_core::{TransportError, MAXIMUM_MESSAGE_LENGTH};

pub fn prepare_message(msg: TransportMessage) -> Result<Vec<u8>> {
    let mut msg_buf = msg.encode().map_err(|_| TransportError::SendBadMessage)?;

    if msg_buf.len() > MAXIMUM_MESSAGE_LENGTH {
        return Err(TransportError::Capacity.into());
    }

    // Create a buffer that includes the message length in big endian
    let mut len = (msg_buf.len() as u16).to_be_bytes().to_vec();

    // Fun fact: reversing a vector in place, appending the length,
    // and then reversing it again is faster for large message sizes
    // than adding the large chunk of data.
    //
    // https://play.rust-lang.org/?version=stable&mode=release&edition=2018&gist=8669a640004ac85c7be38b19e3e73dcb
    msg_buf.reverse();
    len.reverse();
    msg_buf.append(&mut len);
    msg_buf.reverse();

    Ok(msg_buf)
}

#[cfg(test)]
mod test {
    use ockam_core::route;

    use super::{prepare_message, TransportMessage};

    #[test]
    fn prepare_message_should_discard_large_messages() {
        let msg = TransportMessage::v1(route![], route![], vec![0; u16::MAX as usize + 1]);
        let result = prepare_message(msg);
        assert!(result.is_err());
    }
}
