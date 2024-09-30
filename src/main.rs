use core::num;

use rand::seq::SliceRandom;
use rand::Rng;
use std::cmp;
use num_bigint::{BigInt, Sign};
use num_traits::{FromPrimitive, Zero};

mod primes;
mod euclidean;
mod crypto;

//FIXME: Implement efficient exponentiation algorithm, the naive one is too slow
fn encrypt_number(number_to_encrypt: &BigInt, key: &crypto::Key) -> BigInt {
    let modulo = BigInt::from_u64(key.modulo).unwrap();
    let mut exponent = BigInt::from_u64(key.exponent).unwrap();
    let mut result: BigInt = BigInt::from_u8(1).unwrap();
    while !exponent.is_zero() {
        result = (result * number_to_encrypt) % &modulo;
        exponent = exponent - 1;
    }
    result
}

fn encrypt_chunk(data: &Vec<u8>, key: &crypto::Key, block_size_bytes: usize) -> Vec<u8> {
    let number_to_encrypt = BigInt::from_bytes_be(Sign::Plus, data);
    let encrypted = encrypt_number(&number_to_encrypt, key).to_bytes_be();
    let mut result_bytes = encrypted.1;
    while result_bytes.len() < block_size_bytes {
        result_bytes.insert(0, 0u8); // Pad the result with zeros to be exactly block_size_bytes
    }
    result_bytes
}

fn encrypt_bytes(data: &Vec<u8>, key: &crypto::Key) -> Vec<u8> {
    let block_size_bytes = cmp::max(key.modulo.ilog2() / 8, 1) as usize;
    println!("block size = {} bytes", block_size_bytes);
    let prefix = data.len().to_string() + ":";
    let mut all_bytes: Vec<u8> = prefix.as_bytes().to_vec();
    all_bytes.extend(data);

    let mut encrypted: Vec<u8> = Vec::new();
    for chunk in all_bytes.chunks(block_size_bytes) {
        encrypted.extend(encrypt_chunk(&chunk.to_vec(), &key, block_size_bytes));
    }
    encrypted
}

fn decrypt_bytes(data: &Vec<u8>, key: &crypto::Key) -> Vec<u8> {
    let block_size_bytes = cmp::max(key.modulo.ilog2() / 8, 1) as usize;
    println!("block size = {} bytes", block_size_bytes);
    let mut decrypted: Vec<u8> = Vec::new();
    for chunk in data.chunks(block_size_bytes) {
        decrypted.extend(encrypt_chunk(&chunk.to_vec(), &key, block_size_bytes));
    }
    //TODO: Read the length of the encrypted data (in bytes) and 
    decrypted
}

fn main() {
    println!("RSA-like encryption algorithm relying on the Euler number theory theorem will be implemented here...");
    //TODO: This is just an initial version, to be secure the primes would have to be selected randomly from a range  > Math.pow(2, 64) - Math.pow(2, 128)

    //let primes_bottom: usize = 1024; //2^10
    //let primes_top: usize = 16384; //2^24
    let primes_bottom: usize = 262144; //2^18
    let primes_top: usize = 16777216; //2^24

    let mut rng = rand::thread_rng();
    let mut primes_from = rng.gen_range(primes_bottom + 1..primes_top);
    let segment_size = 1000;
    let &p: &usize = primes::primes_segment(primes_from, primes_from + segment_size).choose(&mut rng).unwrap();
    let mut q: usize = p;
    while q == p {
        primes_from = rng.gen_range(primes_bottom + 1..primes_top);
        let &new_q: &usize = primes::primes_segment(primes_from, primes_from + segment_size).choose(&mut rng).unwrap();
        q = new_q;
    }
    let public_exponent: u64 = 65537;

    //Just smaller numbers easier to debug with
    /*
    let public_exponent: u64 = 17;
    let p: u64 = 61;
    let q: u64 = 53;
    */

    let n: usize = p * q;
    let totient_function = (p - 1) * (q - 1);
    println!("p={:?}, q={:?}, n={:?}, totient_function={:?}", p, q, n, totient_function);

    //FIXME: find_private_key (and find_gcd_and_bezout_coefficients used by it) should use BigInt arithmetic, otherwise overflowing leads to the wrong solution
    let private_exponent: u64 = crypto::find_private_key(totient_function as i64, public_exponent as i64) as u64;

    println!("d={:?}, e={:?}", public_exponent, private_exponent);

    let public_key = crypto::Key {
        exponent: public_exponent,
        modulo: n as u64
    };
    let private_key = crypto::Key {
        exponent: private_exponent,
        modulo: n as u64
    };

    let original_number = BigInt::from_u32(65).unwrap();
    println!("original number = {}", original_number);
    let encrypted = encrypt_number(&original_number, &public_key);
    println!("encrypted number = {}", encrypted);
    let decrypted = encrypt_number(&encrypted, &private_key);
    println!("decrypted number = {}", decrypted);
    assert_eq!(original_number, decrypted);

    /*
    let text = "The quick brown fox jumps over the lazy dog";
    let encrypted = encrypt_bytes(&text.as_bytes().to_vec(), &public_key);
    let decrypted = decrypt_bytes(&encrypted, &private_key);
    let decrypted_text = String::from_utf8_lossy(&decrypted);
    println!("Decrypted text: '{}'", decrypted_text)
    */
}

//TODO: Find two large prime numbers p and q. n = p * q, phi(n) = (p - 1)(q - 1)
//TODO: Choose a large enough public exponent e such as e > 10000, e is prime and e < phi(n)
//TODO: Compute the public and the private exponent (components of the public and private keys), use the extended Euclidean algorithm to solve d * e = 1 (mod phi(n))
//TODO: Implement the encryption and decryption procedures

//TODO: Add command line interface
// - Generate public and private keys and store them in some format (base-64 encoded) in two separate files
// - Encrypt Vec<u8> input using the provided key (can be either public or private due to the symmetric nature of the algorithm)
// - Decrypt Vec<u8> input using the provided key (can be either public or private due to the symmetric nature of the algorithm)

//TODO: Add examples:
// - How the command line tool can be used to sign and verify messages (signing with the private key)
// - How the command line tool can be used to decrypt messages sent to the addressee and encrypted with the public key of the addressee

//TODO: Use very large prime numbers, i.e. hundreds of digits.
// - Use AKS primality test starting with a random  number  and crossing out all the multiples of the prime numbers less than 10000(optimization)
// - Use BigInt arithmetic and avoid the limitations of u64 (and avoid u64 <-> i64 conversions especially and ignoring potential overflows)

//TODO: Use a padding scheme when ecrypting the message to increase security
//TODO: Allow to stream the message contents when encrypting and decrypting