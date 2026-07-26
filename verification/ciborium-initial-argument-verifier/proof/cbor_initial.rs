use vstd::prelude::*;

verus! {

pub struct Decoded {
    pub major: u8,
    pub additional: u8,
    pub argument: u64,
    pub consumed: u8,
}

pub enum InitialResult {
    Decoded(Decoded),
    Empty,
    Reserved,
    Truncated,
}

pub open spec fn width_spec(additional: u8) -> u8 {
    if additional < 24 {
        0
    } else if additional == 24 {
        1
    } else if additional == 25 {
        2
    } else if additional == 26 {
        4
    } else {
        8
    }
}

pub open spec fn argument_spec(
    additional: u8,
    byte0: u8,
    byte1: u8,
    byte2: u8,
    byte3: u8,
    byte4: u8,
    byte5: u8,
    byte6: u8,
    byte7: u8,
) -> u64 {
    if additional < 24 {
        additional as u64
    } else if additional == 24 {
        byte0 as u64
    } else if additional == 25 {
        ((byte0 as u64) << 8) | byte1 as u64
    } else if additional == 26 {
        ((byte0 as u64) << 24)
            | ((byte1 as u64) << 16)
            | ((byte2 as u64) << 8)
            | byte3 as u64
    } else {
        ((byte0 as u64) << 56)
            | ((byte1 as u64) << 48)
            | ((byte2 as u64) << 40)
            | ((byte3 as u64) << 32)
            | ((byte4 as u64) << 24)
            | ((byte5 as u64) << 16)
            | ((byte6 as u64) << 8)
            | byte7 as u64
    }
}

fn argument_width(additional: u8) -> (result: u8)
    requires
        additional <= 27,
    ensures
        result == width_spec(additional),
        result <= 8,
{
    if additional < 24 {
        0
    } else if additional == 24 {
        1
    } else if additional == 25 {
        2
    } else if additional == 26 {
        4
    } else {
        8
    }
}

fn decode_argument(
    additional: u8,
    byte0: u8,
    byte1: u8,
    byte2: u8,
    byte3: u8,
    byte4: u8,
    byte5: u8,
    byte6: u8,
    byte7: u8,
) -> (result: u64)
    requires
        additional <= 27,
    ensures
        result == argument_spec(
            additional,
            byte0,
            byte1,
            byte2,
            byte3,
            byte4,
            byte5,
            byte6,
            byte7,
        ),
{
    if additional < 24 {
        additional as u64
    } else if additional == 24 {
        byte0 as u64
    } else if additional == 25 {
        ((byte0 as u64) << 8) | byte1 as u64
    } else if additional == 26 {
        ((byte0 as u64) << 24)
            | ((byte1 as u64) << 16)
            | ((byte2 as u64) << 8)
            | byte3 as u64
    } else {
        ((byte0 as u64) << 56)
            | ((byte1 as u64) << 48)
            | ((byte2 as u64) << 40)
            | ((byte3 as u64) << 32)
            | ((byte4 as u64) << 24)
            | ((byte5 as u64) << 16)
            | ((byte6 as u64) << 8)
            | byte7 as u64
    }
}

fn decode_initial(
    prefix_present: bool,
    prefix: u8,
    available_argument_bytes: u8,
    byte0: u8,
    byte1: u8,
    byte2: u8,
    byte3: u8,
    byte4: u8,
    byte5: u8,
    byte6: u8,
    byte7: u8,
) -> (result: InitialResult)
    requires
        prefix_present ==> (prefix & 31) <= 30,
    ensures
        match result {
            InitialResult::Empty => !prefix_present,
            InitialResult::Reserved => {
                &&& prefix_present
                &&& 28 <= (prefix & 31) <= 30
            },
            InitialResult::Truncated => {
                &&& prefix_present
                &&& (prefix & 31) <= 27
                &&& available_argument_bytes < width_spec(prefix & 31)
            },
            InitialResult::Decoded(decoded) => {
                &&& prefix_present
                &&& (prefix & 31) <= 27
                &&& available_argument_bytes >= width_spec(prefix & 31)
                &&& decoded.major == prefix >> 5
                &&& decoded.major <= 7
                &&& decoded.additional == prefix & 31
                &&& decoded.argument == argument_spec(
                    prefix & 31,
                    byte0,
                    byte1,
                    byte2,
                    byte3,
                    byte4,
                    byte5,
                    byte6,
                    byte7,
                )
                &&& decoded.consumed == 1 + width_spec(prefix & 31)
                &&& decoded.consumed <= 9
            },
        },
{
    if !prefix_present {
        return InitialResult::Empty;
    }

    let additional = prefix & 31;
    if additional >= 28 {
        return InitialResult::Reserved;
    }

    let width = argument_width(additional);
    if available_argument_bytes < width {
        return InitialResult::Truncated;
    }

    assert((prefix >> 5) <= 7) by (bit_vector);
    let major = prefix >> 5;
    let argument = decode_argument(
        additional,
        byte0,
        byte1,
        byte2,
        byte3,
        byte4,
        byte5,
        byte6,
        byte7,
    );
    InitialResult::Decoded(Decoded {
        major,
        additional,
        argument,
        consumed: 1 + width,
    })
}

fn main() {
    assert((0x1bu8 & 31u8) == 27u8) by (bit_vector);
    assert((0x1bu8 >> 5) == 0u8) by (bit_vector);
    assert(argument_spec(27, 1, 2, 3, 4, 5, 6, 7, 8) == 0x0102_0304_0506_0708u64)
        by (bit_vector);
    match decode_initial(
        true,
        0x1b,
        8,
        1,
        2,
        3,
        4,
        5,
        6,
        7,
        8,
    ) {
        InitialResult::Decoded(decoded) => {
            assert(decoded.major == 0);
            assert(decoded.additional == 27);
            assert(decoded.argument == 0x0102_0304_0506_0708);
            assert(decoded.consumed == 9);
        },
        _ => assert(false),
    }

    assert((0x5au8 & 31u8) == 26u8) by (bit_vector);
    match decode_initial(true, 0x5a, 3, 0, 0, 0, 0, 0, 0, 0, 0) {
        InitialResult::Truncated => {},
        _ => assert(false),
    }

    assert((0x1cu8 & 31u8) == 28u8) by (bit_vector);
    match decode_initial(true, 0x1c, 0, 0, 0, 0, 0, 0, 0, 0, 0) {
        InitialResult::Reserved => {},
        _ => assert(false),
    }

    match decode_initial(false, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0) {
        InitialResult::Empty => {},
        _ => assert(false),
    }
}

}
