use *;
use openssl::rand;
use openssl::rsa::{Rsa, PKCS1_PADDING};
use openssl::symm::{self, Cipher};

pub fn encrypt(rsa: &Rsa, input: &Packet) -> Result<Vec<u8>, Box<::std::error::Error>> {
    let encoded = serialize(input)?;

    let mut key = vec![0; 32];
    let mut iv = vec![0; 16];

    rand::rand_bytes(&mut key)?;
    rand::rand_bytes(&mut iv)?;

    let mut encrypted_aes = symm::encrypt(Cipher::aes_256_cbc(), &key, Some(&iv), &encoded)?;
    let size_aes = encrypted_aes.len();

    let size_rsa = rsa.size();
    let mut encrypted_rsa = vec![0; size_rsa];

    key.append(&mut iv);

    rsa.public_encrypt(&key, &mut encrypted_rsa, PKCS1_PADDING)?;

    let mut encrypted = Vec::with_capacity(4+size_rsa+size_aes);
    encrypted.push((size_rsa >> 8)  as u8);
    encrypted.push((size_rsa % 256) as u8);
    encrypted.push((size_aes >> 8)  as u8);
    encrypted.push((size_aes % 256) as u8);
    encrypted.append(&mut encrypted_rsa);
    encrypted.append(&mut encrypted_aes);

    Ok(encrypted)
}

pub fn decrypt(rsa: &Rsa, size_rsa: usize, input: &[u8]) -> Result<Packet, Box<::std::error::Error>> {
    let mut keyiv = vec![0; size_rsa];
    rsa.private_decrypt(&input[..size_rsa], &mut keyiv, PKCS1_PADDING)?;
    keyiv.truncate(32+16);

    let (key, iv) = keyiv.split_at(32);
    let decrypted = symm::decrypt(Cipher::aes_256_cbc(), key, Some(iv), &input[size_rsa..])?;

    Ok(deserialize(&decrypted)?)
}
