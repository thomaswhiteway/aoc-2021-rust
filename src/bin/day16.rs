use bitreader::BitReader;
use std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Debug)]
enum Payload {
    Literal(u64),
    Sum(Box<[Packet]>),
    Product(Box<[Packet]>),
    Minimum(Box<[Packet]>),
    Maximum(Box<[Packet]>),
    GreaterThan(Box<[Packet]>),
    LessThan(Box<[Packet]>),
    EqualTo(Box<[Packet]>),
}

impl Payload {
    fn evaluate(&self) -> u64 {
        use Payload::*;
        match self {
            Literal(val) => *val,
            Sum(packets) => packets.iter().map(Packet::evaluate).sum(),
            Product(packets) => packets.iter().map(Packet::evaluate).product(),
            Minimum(packets) => packets.iter().map(Packet::evaluate).min().unwrap(),
            Maximum(packets) => packets.iter().map(Packet::evaluate).max().unwrap(),
            GreaterThan(packets) => {
                if packets[0].evaluate() > packets[1].evaluate() {
                    1
                } else {
                    0
                }
            }
            LessThan(packets) => {
                if packets[0].evaluate() < packets[1].evaluate() {
                    1
                } else {
                    0
                }
            }
            EqualTo(packets) => {
                if packets[0].evaluate() == packets[1].evaluate() {
                    1
                } else {
                    0
                }
            }
        }
    }
}

#[derive(Debug)]
struct Packet {
    version: u8,
    payload: Payload,
}

impl Packet {
    fn evaluate(&self) -> u64 {
        self.payload.evaluate()
    }

    fn total_version(&self) -> usize {
        use Payload::*;
        self.version as usize
            + match &self.payload {
                Literal(_) => 0,
                Sum(packets) | Product(packets) | Minimum(packets) | Maximum(packets)
                | GreaterThan(packets) | LessThan(packets) | EqualTo(packets) => {
                    packets.iter().map(Packet::total_version).sum()
                }
            }
    }
}

fn read_data<P: AsRef<Path>>(input: P) -> Box<[u8]> {
    let mut data = fs::read_to_string(input).unwrap();
    if data.ends_with('\n') {
        data.pop();
    }
    if data.len() % 2 == 1 {
        data.push('0');
    }

    hex::decode(&data).unwrap().into_boxed_slice()
}

fn read_literal_payload(reader: &mut BitReader) -> Payload {
    let mut value = 0_u64;

    loop {
        let next = reader.read_u64(5).unwrap();
        value <<= 4;
        value |= next & 0xF;

        if next & 0x10 == 0 {
            break;
        }
    }

    Payload::Literal(value)
}

fn read_bits(reader: &mut BitReader, length: u64) -> Box<[u8]> {
    let mut bytes = vec![];
    let mut remaining = length;
    let mut subreader = reader.relative_reader();

    while remaining > 0 {
        let value = if remaining >= 8 {
            subreader.read_u8(8).unwrap()
        } else {
            subreader.read_u8(remaining as u8).unwrap() << (8 - remaining)
        };

        bytes.push(value);

        remaining = remaining.saturating_sub(8);
    }

    reader.skip(length).unwrap();

    bytes.into_boxed_slice()
}

fn read_defined_length_packets(reader: &mut BitReader) -> Box<[Packet]> {
    let length = reader.read_u64(15).unwrap();
    let data = read_bits(reader, length);
    let mut subreader = BitReader::new(&data);

    let mut packets = vec![];
    while subreader.remaining() >= 8 {
        packets.push(read_packet(&mut subreader).unwrap());
    }

    packets.into_boxed_slice()
}

fn read_defined_num_packets(reader: &mut BitReader) -> Box<[Packet]> {
    let num_packets = reader.read_u16(11).unwrap();

    let mut packets = vec![];
    for _ in 0..num_packets {
        packets.push(read_packet(reader).unwrap());
    }

    packets.into_boxed_slice()
}

fn read_operator_payload<F>(reader: &mut BitReader, cons: F) -> Payload
where
    F: Fn(Box<[Packet]>) -> Payload,
{
    let packets = read_sub_packets(reader);
    cons(packets)
}

fn read_sub_packets(reader: &mut BitReader) -> Box<[Packet]> {
    let length_type = reader.read_u8(1).unwrap();

    if length_type == 0 {
        read_defined_length_packets(reader)
    } else {
        read_defined_num_packets(reader)
    }
}

fn read_packet(reader: &mut BitReader) -> Option<Packet> {
    if reader.remaining() < 8 {
        return None;
    }

    let version = reader.read_u8(3).unwrap();
    let type_id = reader.read_u8(3).unwrap();

    use Payload::*;
    let payload = match type_id {
        0 => read_operator_payload(reader, Sum),
        1 => read_operator_payload(reader, Product),
        2 => read_operator_payload(reader, Minimum),
        3 => read_operator_payload(reader, Maximum),
        4 => read_literal_payload(reader),
        5 => read_operator_payload(reader, GreaterThan),
        6 => read_operator_payload(reader, LessThan),
        7 => read_operator_payload(reader, EqualTo),
        _ => panic!("Unknown type ID {}", type_id),
    };

    Some(Packet { version, payload })
}

fn parse_packets(data: &[u8]) -> Box<[Packet]> {
    let mut reader = BitReader::new(data);
    let mut packets = vec![];

    while let Some(packet) = read_packet(&mut reader) {
        packets.push(packet);
    }

    packets.into_boxed_slice()
}

fn count_total_versions(packets: &[Packet]) -> usize {
    packets.iter().map(Packet::total_version).sum()
}

fn main() {
    let opt = Opt::from_args();

    let data = read_data(opt.input);
    let packets = parse_packets(&data);
    let total_version = count_total_versions(&packets);
    println!("{}", total_version);
    println!("{}", packets[0].evaluate());
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_one() {
        let data = hex::decode("8A004A801A8002F478").unwrap();
        let packets = parse_packets(&data);
        let total_version = count_total_versions(&packets);
        assert_eq!(total_version, 16);
    }

    #[test]
    fn test_two() {
        let data = hex::decode("620080001611562C8802118E34").unwrap();
        let packets = parse_packets(&data);
        let total_version = count_total_versions(&packets);
        assert_eq!(total_version, 12);
    }

    #[test]
    fn test_three() {
        let data = hex::decode("C0015000016115A2E0802F182340").unwrap();
        let packets = parse_packets(&data);
        let total_version = count_total_versions(&packets);
        assert_eq!(total_version, 23);
    }

    #[test]
    fn test_four() {
        let data = hex::decode("A0016C880162017C3686B18A3D4780").unwrap();
        let packets = parse_packets(&data);
        let total_version = count_total_versions(&packets);
        assert_eq!(total_version, 31);
    }

    #[test]
    fn test_parse_literal() {
        let data = hex::decode("D2FE28").unwrap();
        parse_packets(&data);
    }
}
