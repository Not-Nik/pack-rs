use std::io::{Write, Result, Read};
use rustvarints::*;
use ruststreams::Stream;
use libflate::zlib::{Encoder, Decoder};

#[cfg(test)]
mod tests {
    use crate::{Packet, PacketWrite, PacketRead};
    use rustvarints::{VarWrite, VarRead};
    use ruststreams::Stream;

    fn test_on_stream(comp: bool) {
        let mut stream = Stream::new();

        let mut pack_in = Packet::new(42);
        for i in 10..20 {
            pack_in.write_var_int(i).expect("Write data");
        }
        stream.write_packet(&pack_in, comp).expect("Write packet");

        let mut pack_out = stream.read_packet(comp).expect("Read packet");

        assert_eq!(pack_in.id, pack_out.id);
        for i in 10..20 {
            assert_eq!(pack_out.read_var_int().expect("Read data"), i);
        }
    }

    #[test]
    fn test_uncompressed() {
        test_on_stream(false);
    }

    #[test]
    fn test_compressed() {
        test_on_stream(true);
    }

    #[test]
    fn readme_write() {
        let mut my_stream = Stream::new();

        let mut packet = Packet::new(1);
        packet.write_var_int(42);

        my_stream.write_packet(&packet, false);
    }

    #[test]
    fn readme_read() {
        let mut my_stream = Stream::new();

        let packet = my_stream.read_packet(false);
    }
}

pub struct Packet {
    pub id: i32,
    data: Stream<u8>,
}

pub trait PacketWrite {
    fn write_packet(&mut self, pack: &Packet, compression: bool) -> Result<usize>;
}

pub trait PacketRead {
    fn read_packet(&mut self, comp: bool) -> Result<Packet>;
}

impl Packet {
    pub fn new(id: i32) -> Packet {
        Packet { id, data: Stream::new() }
    }
}

impl Read for Packet {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.data.read(buf)
    }
}

impl Write for Packet {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.data.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.data.flush()
    }
}

impl<T> PacketWrite for T
    where T: Write {
    fn write_packet(&mut self, pack: &Packet, compression: bool) -> Result<usize> {
        let data_length = get_var_int_size(pack.id) + pack.data.as_slice().len();

        // TODO: Add results together

        if compression {
            let mut encoder = Encoder::new(Vec::new()).unwrap();

            encoder.write_var_int(pack.id)?;
            encoder.write(pack.data.as_slice())?;

            let data = encoder.finish().into_result()?;

            let packet_length = get_var_int_size(data_length as i32) + data.len();

            self.write_var_int(packet_length as i32)?;
            self.write_var_int(data_length as i32)?;
            self.write(data.as_slice())?;
        } else {
            self.write_var_int(data_length as i32)?;
            self.write_var_int(pack.id)?;
            self.write(pack.data.as_slice())?;
        }
        // TODO: lol wth is this (Add results together)
        Ok(0)
    }
}

impl<T> PacketRead for T
    where T: Read {
    fn read_packet(&mut self, comp: bool) -> Result<Packet> {
        let mut res = Packet::new(0);

        let packet_length = self.read_var_int()? as usize;

        if comp {
            let _ = self.read_var_int()? as usize;

            let mut buf = vec![0; packet_length - get_var_int_size(res.id)];
            self.read(buf.as_mut_slice())?;

            // TODO: find some way to write to data directly without exposing data
            let mut decoder = Decoder::new(&buf[..])?;
            let mut decoded_data = Vec::new();
            decoder.read_to_end(&mut decoded_data)?;

            res.write(decoded_data.as_slice())?;

            res.id = res.read_var_int()?;
        } else {
            res.id = self.read_var_int()?;

            // TODO: find some way to write to data directly without exposing data
            let mut buf = vec![0; packet_length - get_var_int_size(res.id)];
            self.read(buf.as_mut_slice())?;

            res.write(buf.as_slice())?;
        }

        Ok(res)
    }
}
