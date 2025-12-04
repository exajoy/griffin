use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures_core::Stream;
use http_body::Frame;
use http_body_util::StreamBody;
use prost::Message;

use futures_util::StreamExt;
use prost::DecodeError;

//collect protobuf messages from stream body
pub async fn collect_messages<M>(
    mut body: StreamBody<impl Stream<Item = Result<Frame<Bytes>, hyper::Error>> + Send + Unpin>,
) -> Result<Vec<M>, DecodeError>
where
    M: Message + Default,
{
    let mut messages = Vec::new();
    let mut buf = BytesMut::new();

    while let Some(next) = body.next().await {
        let frame = next.expect("body frame");

        // data frame consumption
        if let Ok(data) = frame.into_data() {
            // collect data into buffer
            buf.extend_from_slice(&data);
            while buf.len() >= 5 {
                let mut reader = &buf[..];

                //https://datatracker.ietf.org/doc/html/rfc7540#section-4.1
                //TODO: support compression
                let _compressed_flag = reader.get_u8();
                // Length: 24-bit, type =8-bit
                let msg_len = reader.get_u32() as usize;

                if buf.len() < 5 + msg_len {
                    // incomplete message — wait for more data
                    break;
                }

                // collect message bytes after getting message length
                // by ignoring the first 5 bytes
                let msg_bytes = &buf[5..5 + msg_len];
                // bytes to message
                let msg = M::decode(msg_bytes)?;

                // add new message to list
                messages.push(msg);

                // remove consumed bytes
                buf.advance(5 + msg_len);
            }
        }
    }

    Ok(messages)
}

// convert message to frame
pub fn message_to_frame(message: &impl Message) -> BytesMut {
    let mut buf = BytesMut::new();
    message.encode(&mut buf).unwrap();

    let mut framed = BytesMut::new();
    //TODO support compression
    framed.put_u8(0); // 0: not compressed
    framed.put_u32(buf.len() as u32);
    framed.put_slice(&buf);
    framed
}
