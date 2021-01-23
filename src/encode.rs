const BASE64_TABLE: [u8; 64] = [
    b'A', b'B', b'C', b'D', b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O', b'P',
    b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X', b'Y', b'Z', b'a', b'b', b'c', b'd', b'e', b'f',
    b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v',
    b'w', b'x', b'y', b'z', b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'+', b'/',
];
const PAD_BYTE: u8 = b'=';

//Encodes arbitrary octets as Base64
//Returns a string.
pub fn encode<T: AsRef<[u8]>>(input: T) -> String {
    let len = encode_size(input.as_ref().len());
    let mut buf = vec![0u8; len];

    encode_with_padding(input.as_ref(), &mut buf, len);

    String::from_utf8(buf).expect("Invalid UTF8")
}

//Encode arbitrary octets as Base64.
//Writes into the given buffer.
//It is useful for writing to pre-allocated memory like in the stack.
pub fn encode_slice<T: AsRef<[u8]>>(input: T, output: &mut [u8]) -> usize {
    let input = input.as_ref();
    
    let encode_size = encode_size(input.len());
    let mut b64_output = &mut output[..encode_size];//only required amount of space in buffer is used obviously

    encode_with_padding(input, &mut b64_output, encode_size);

    encode_size
}

//this helper function combines the two functions encode_to_slice() and add_padding().
//writes into the supplied output buffer whose length must be equal to the size of encoded input.
//encoded_size is the size calculated for input.
//
fn encode_with_padding(input: &[u8], output: &mut [u8], encoded_size: usize) {
    debug_assert_eq!(output.len(), encoded_size);

    let b64_bytes = encode_to_slice(input, output);
    let padding_bytes = add_padding( input.len(), &mut output[b64_bytes..]);

    let encoded_bytes = b64_bytes + padding_bytes;
    debug_assert_eq!(encoded_bytes, encoded_size);
}

// caluclate size of base64 string including padding
pub fn encode_size(input_len: usize) -> usize {
    let rem = input_len % 3;
    let input_chunks_complete = input_len / 3;
    let complete_output_chunks = input_chunks_complete * 4;

    if rem > 0 {
        // padding included
        complete_output_chunks + 4
    } else {
        complete_output_chunks
    }
}

fn read_u32(s: &[u8]) -> u32 {
    let temp = [0, s[0], s[1], s[2] ];
    u32::from_be_bytes( temp)
}

//encodes input to base64 bytes
//output must be long enough to hold the encoded 'input' without padding
//Returns the number of bytes written
pub fn encode_to_slice(input: &[u8], output: &mut [u8]) -> usize {
    let mut input_index: usize = 0;
    let mut output_index: usize = 0;

    const LOW_SIX_BITS: u32 = 0x3f;
    const LOW_SIX_BITS_U8: u8 = 0x3f;

    let rem = input.len() % 3;
    let last_index = input.len() - rem;

    while input_index < last_index {

        //read 3 bytes into u32
        let input_chunk = read_u32(&input[input_index..(input_index + 3) ]);
        let output_chunk = &mut output[output_index..(output_index + 4) ];

        output_chunk[0] = BASE64_TABLE[ ( (input_chunk >> 18) & LOW_SIX_BITS) as  usize];
        output_chunk[1] = BASE64_TABLE[ ( (input_chunk >> 12) & LOW_SIX_BITS) as  usize];
        output_chunk[2] = BASE64_TABLE[ ( (input_chunk >> 6) & LOW_SIX_BITS) as  usize];
        output_chunk[3] = BASE64_TABLE[ ( (input_chunk >> 0) & LOW_SIX_BITS) as  usize];

        input_index += 3;
        output_index += 4;
    }

    if rem == 2 {
        let output_chunk = &mut output[output_index..(output_index + 3)];

        output_chunk[0] = BASE64_TABLE[ ((input[last_index] >> 2) & LOW_SIX_BITS_U8) as usize];
        output_chunk[1] = BASE64_TABLE[ (((input[last_index] << 4) 
                | (input[last_index + 1] >> 4) ) 
                & LOW_SIX_BITS_U8) as usize];
        output_chunk[2] = BASE64_TABLE[ ((input[last_index + 1] << 2) & LOW_SIX_BITS_U8) as usize];

        output_index += 3;
    } else if rem == 1 {
        let output_chunk = &mut output[output_index..(output_index + 2)];

        output_chunk[0] = BASE64_TABLE[ ((input[last_index] >> 2) & LOW_SIX_BITS_U8) as usize];
        output_chunk[1] = BASE64_TABLE[ ((input[last_index] << 4) & LOW_SIX_BITS_U8) as usize];

        output_index += 2;
    }

    output_index
}

//writes padding bytes to output
//Returns number of bytes written.
//
//output must be of length at least 2.
pub fn add_padding( input_len: usize, output: &mut [u8] ) -> usize {
    let rem = input_len % 3;
    let len = (3 - rem) % 3;

    for i in 0..len {
        output[i] = PAD_BYTE;
    }

    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_size() {
        let input = include_bytes!("plain.txt");
        let output = include_bytes!("encoded.txt");

        let in_len = input.len();
        let out_len = output.len();

        assert_eq!(out_len, encode_size(in_len));
    }

    #[test]
    fn check_encode_with_padding() {
        let input = include_bytes!("plain.txt");
        let output = &include_bytes!("encoded.txt")[..];

        let in_len = input.len();
        let out_len = output.len();

        let mut buf = vec![0u8; encode_size(in_len)];
        encode_with_padding(input, &mut buf, out_len);

        assert_eq!(output, buf);
    }
}
