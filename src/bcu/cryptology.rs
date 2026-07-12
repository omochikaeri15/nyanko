mod ciphers;
mod parser;

use std::error;
use std::fmt;

use serde::Deserialize;

const HEAD_DATA: &[u8] = b"battlecatsultimate";

/// Represents cryptographic and parsing errors encountered when processing BCU packs.
///
/// This enum covers various failure states that can occur during the validation,
/// decryption, and parsing phases of extracting data from a Battle Cats Ultimate pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    InvalidHeader,
    InvalidLength,
    InvalidKeyLength,
    DecryptionFailed,
    PaddingError,
    InvalidJson,
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHeader => write!(formatter, "Invalid BCU header signature"),
            Self::InvalidLength => write!(formatter, "Data slice is too short or malformed"),
            Self::InvalidKeyLength => write!(formatter, "Provided key must be exactly 16 bytes"),
            Self::DecryptionFailed => write!(formatter, "AES decryption failed"),
            Self::PaddingError => write!(formatter, "Block padding evaluation failed"),
            Self::InvalidJson => write!(formatter, "Decrypted JSON metadata is malformed"),
        }
    }
}

impl error::Error for Error {}

/// Represents metadata for a single file contained within a BCU pack.
///
/// This structure is mapped directly from the decrypted JSON manifest embedded
/// inside the BCU pack, detailing the expected internal path and exact byte size of the file.
#[derive(Deserialize)]
pub struct FileDescriptor {
    pub path: String,
    pub size: usize,
}

/// Represents the JSON manifest structure of a BCU pack.
///
/// Contains a sequential list of file descriptors that map out the boundaries
/// and original sizes of the encrypted data chunks that immediately follow the manifest.
#[derive(Deserialize)]
pub struct PackDescriptor {
    pub files: Vec<FileDescriptor>,
}

/// Represents a fully decrypted and extracted file from a BCU pack.
///
/// Holds the internal virtual file path and the raw, unpadded byte data
/// of the extracted file, ready for processing or saving to disk.
pub struct ExtractedFile {
    pub path: String,
    pub data: Vec<u8>,
}

/// Represents a fully decrypted BCU pack.
///
/// This structure acts as the container for the parsed pack metadata and all
/// successfully extracted file contents held in memory.
pub struct Pack {
    pub metadata: PackDescriptor,
    pub files: Vec<ExtractedFile>,
}

/// Extracts the internal authentication checksum used for validating a pack.
///
/// Parses the header of a BCU pack (or multi-pack container) to isolate the 16-byte
/// checksum. This hash is utilized by the client to verify password correctness
/// prior to authorizing the decryption sequence.
///
/// # Arguments
/// * `data` - A byte slice representing the raw file data.
/// * `is_multi` - A boolean indicating whether the data is a standard pack (`false`) or a multi-pack container (`true`).
///
/// # Returns
/// A `Result` containing the extracted 16-byte checksum array on success, or an `Error`
/// if the header signature is invalid or the byte slice is truncated.
pub fn extract_password_checksum(data: &[u8], is_multi: bool) -> Result<[u8; 16], Error> {
    let expected_header = ciphers::md5_hash(HEAD_DATA);

    let (header, remainder) = parser::take_array::<16>(data)?;
    if header != expected_header {
        return Err(Error::InvalidHeader);
    }

    if !is_multi {
        let (hash, _remainder) = parser::take_array::<16>(remainder)?;
        return Ok(hash);
    }

    let (json_size, remainder) = parser::take_u32(remainder)?;
    let (_json_data, remainder) = parser::take_bytes(remainder, json_size as usize)?;

    let (inner_header, remainder) = parser::take_array::<16>(remainder)?;
    if inner_header != expected_header {
        return Err(Error::InvalidHeader);
    }

    let (hash, _remainder) = parser::take_array::<16>(remainder)?;

    Ok(hash)
}

/// Derives a 16-byte AES decryption key from a plaintext password string.
///
/// Battle Cats Ultimate generates its encryption keys by hashing the user's password.
/// If the pack has no password, this function gracefully handles an empty string
/// to generate the default cryptographic lock key.
///
/// # Arguments
/// * `password` - A string slice representing the user's password.
///
/// # Returns
/// A 16-byte array ready to be passed directly into the decryption engine.
pub fn derive_decryption_key(password: &str) -> [u8; 16] {
    ciphers::md5_hash(password.as_bytes())
}

/// Decrypts a contiguous BCU pack stream into memory.
///
/// Validates the pack header, decrypts the embedded JSON manifest to determine file
/// boundaries, and sequentially decrypts each file chunk using AES-128-CBC. The
/// output size of each chunk is truncated to its exact original byte length to strip
/// any block cipher padding.
///
/// # Arguments
/// * `data` - A byte slice containing the encrypted raw BCU pack data.
/// * `key` - A reference to a 16-byte array used as the AES-128 decryption key.
///
/// # Returns
/// A `Result` containing the fully processed `Pack` structure on success, or an `Error`
/// if cryptographic operations, padding validation, or JSON deserialization fail.
pub fn decrypt(data: &[u8], key: &[u8; 16]) -> Result<Pack, Error> {
    let expected_header = ciphers::md5_hash(HEAD_DATA);
    let expected_iv = ciphers::md5_hash(HEAD_DATA);

    let (header, remainder) = parser::take_array::<16>(data)?;
    if header != expected_header {
        return Err(Error::InvalidHeader);
    }

    let (_hash, remainder) = parser::take_array::<16>(remainder)?;

    let (desc_size, remainder) = parser::take_u32(remainder)?;
    let desc_size_usize = desc_size as usize;
    let enc_desc_size = ciphers::regulate(desc_size_usize);

    let (enc_desc, mut stream) = parser::take_bytes(remainder, enc_desc_size)?;

    let mut dec_desc = ciphers::decrypt_chunk(enc_desc, key, &expected_iv)?;
    dec_desc.truncate(desc_size_usize);

    let Ok(metadata) = serde_json::from_slice::<PackDescriptor>(&dec_desc) else {
        return Err(Error::InvalidJson);
    };

    let mut files = Vec::with_capacity(metadata.files.len());

    for descriptor in &metadata.files {
        let enc_size = ciphers::regulate(descriptor.size);

        let (enc_chunk, next_stream) = parser::take_bytes(stream, enc_size)?;
        stream = next_stream;

        let mut dec_chunk = ciphers::decrypt_chunk(enc_chunk, key, &expected_iv)?;
        dec_chunk.truncate(descriptor.size);

        files.push(ExtractedFile {
            path: descriptor.path.clone(),
            data: dec_chunk,
        });
    }

    Ok(Pack {
        metadata,
        files,
    })
}