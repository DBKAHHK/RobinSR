use std::io;

use tokio::io::{AsyncReadExt, AsyncWriteExt};

const HEAD_MAGIC: [u8; 4] = [0x9d, 0x74, 0xc7, 0x14];
const TAIL_MAGIC: [u8; 4] = [0xd7, 0xa1, 0x52, 0xc8];

pub struct Packet {
    pub cmd: u16,
    pub head: Vec<u8>,
    pub body: Vec<u8>,
}

impl Packet {
    pub async fn read_from<R: AsyncReadExt + Unpin>(reader: &mut R) -> io::Result<Self> {
        let mut header = [0u8; 12];
        reader.read_exact(&mut header).await?;

        if header[0..4] != HEAD_MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid head magic"));
        }

        let cmd = u16::from_be_bytes([header[4], header[5]]);
        let head_len = u16::from_be_bytes([header[6], header[7]]) as usize;
        let body_len = u32::from_be_bytes([header[8], header[9], header[10], header[11]]) as usize;

        let mut payload = vec![0u8; head_len + body_len + 4];
        reader.read_exact(&mut payload).await?;

        let head = payload[0..head_len].to_vec();
        let body = payload[head_len..head_len + body_len].to_vec();
        let tail = &payload[head_len + body_len..];
        if tail != TAIL_MAGIC {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid tail magic"));
        }

        Ok(Self { cmd, head, body })
    }

    pub fn encode(&self) -> Vec<u8> {
        let head_len = self.head.len();
        let body_len = self.body.len();

        let mut out = Vec::with_capacity(12 + head_len + body_len + 4);
        out.extend_from_slice(&HEAD_MAGIC);
        out.extend_from_slice(&self.cmd.to_be_bytes());
        out.extend_from_slice(&(head_len as u16).to_be_bytes());
        out.extend_from_slice(&(body_len as u32).to_be_bytes());
        out.extend_from_slice(&self.head);
        out.extend_from_slice(&self.body);
        out.extend_from_slice(&TAIL_MAGIC);
        out
    }
}

pub struct Connection {
    reader: tokio::net::tcp::OwnedReadHalf,
    writer: tokio::net::tcp::OwnedWriteHalf,
}

impl Connection {
    pub fn new(
        reader: tokio::net::tcp::OwnedReadHalf,
        writer: tokio::net::tcp::OwnedWriteHalf,
    ) -> Self {
        Self { reader, writer }
    }

    pub async fn read_packet(&mut self) -> io::Result<Packet> {
        Packet::read_from(&mut self.reader).await
    }

    pub async fn send_raw(&mut self, cmd: u16, body: &[u8]) -> io::Result<()> {
        let packet = Packet {
            cmd,
            head: Vec::new(),
            body: body.to_vec(),
        };
        self.writer.write_all(&packet.encode()).await
    }

    pub async fn send_empty(&mut self, cmd: u16) -> io::Result<()> {
        self.send_raw(cmd, &[]).await
    }

    pub async fn close(&mut self) -> io::Result<()> {
        self.writer.shutdown().await
    }
}
