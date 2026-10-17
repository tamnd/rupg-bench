//! SCRAM-SHA-256 of RFC 5802 and RFC 7677 for the client, as `libpq` does it with `channel_binding=disable`.
//!
//! The client sends the GS2 header `n,,` and an empty user name, because PostgreSQL takes the user from the startup message. The password is used as it is, without SASLprep, so a password with characters outside ASCII can fail where `libpq` would log in.

use std::num::NonZeroU32;
use std::sync::{Mutex, PoisonError};

use ring::rand::SecureRandom;
use ring::{digest, hmac, pbkdf2, rand};

/// The salted passwords of the last exchanges, by password, salt and iteration count. RFC 5802 lets a client keep them, and the cache keeps the PBKDF2 of the client out of the connection rate, so the rate measures the server.
static SALTED: Mutex<Vec<Salted>> = Mutex::new(Vec::new());

/// A password, a salt, an iteration count and the salted password.
type Salted = (String, Vec<u8>, NonZeroU32, [u8; 32]);

/// The salted password, from the cache or from PBKDF2.
fn salted(password: &str, mix: &[u8], rounds: NonZeroU32) -> [u8; 32] {
    let mut cache = SALTED.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some((.., out)) =
        cache.iter().find(|(p, m, r, _)| p == password && m == mix && *r == rounds)
    {
        return *out;
    }
    let mut out = [0u8; 32];
    pbkdf2::derive(pbkdf2::PBKDF2_HMAC_SHA256, rounds, mix, password.as_bytes(), &mut out);
    if cache.len() >= 16 {
        cache.remove(0);
    }
    cache.push((password.to_owned(), mix.to_vec(), rounds, out));
    out
}

/// The name of the mechanism in `AuthenticationSASL` and `SASLInitialResponse`.
pub(crate) const MECHANISM: &str = "SCRAM-SHA-256";

/// The state of the client between its first message and the last message of the server.
#[derive(Debug)]
pub(crate) struct Client {
    password: String,
    first_bare: String,
    client_part: String,
    /// The `ServerSignature` that the last message of the server must give.
    expected: Option<Vec<u8>>,
}

impl Client {
    /// A client with 18 random bytes as its part of the `r` attribute, as `libpq` does.
    pub(crate) fn new(password: &str) -> Result<Client, String> {
        let mut bytes = [0u8; 18];
        rand::SystemRandom::new()
            .fill(&mut bytes)
            .map_err(|_| "the system gave no random bytes".to_owned())?;
        Ok(Client::with_part(password, "", &encode(&bytes)))
    }

    fn with_part(password: &str, user: &str, client_part: &str) -> Client {
        Client {
            password: password.to_owned(),
            first_bare: format!("n={user},r={client_part}"),
            client_part: client_part.to_owned(),
            expected: None,
        }
    }

    /// The data of `SASLInitialResponse`: the GS2 header and the first message.
    pub(crate) fn first(&self) -> String {
        format!("n,,{}", self.first_bare)
    }

    /// The data of `SASLResponse` for the data of `AuthenticationSASLContinue`.
    pub(crate) fn last(&mut self, server_first: &str) -> Result<String, String> {
        let mut combined = None;
        let mut mix = None;
        let mut rounds = None;
        for attr in server_first.split(',') {
            match attr.split_once('=') {
                Some(("r", v)) => combined = Some(v),
                Some(("s", v)) => mix = Some(decode(v)?),
                Some(("i", v)) => rounds = v.parse::<u32>().ok().and_then(NonZeroU32::new),
                _ => {}
            }
        }
        let bad = || format!("the first message of the server is not valid: {server_first:?}");
        let combined = combined.ok_or_else(bad)?;
        let mix = mix.ok_or_else(bad)?;
        let rounds = rounds.ok_or_else(bad)?;
        if !combined.starts_with(&self.client_part) || combined.len() == self.client_part.len() {
            return Err(bad());
        }
        let salted = salted(&self.password, &mix, rounds);
        let mac = |secret: &[u8], text: &[u8]| {
            hmac::sign(&hmac::Key::new(hmac::HMAC_SHA256, secret), text).as_ref().to_vec()
        };
        let client_secret = mac(&salted, b"Client Key");
        let stored = digest::digest(&digest::SHA256, &client_secret);
        let without_proof = format!("c=biws,r={combined}");
        let message = format!("{},{server_first},{without_proof}", self.first_bare);
        let signature = mac(stored.as_ref(), message.as_bytes());
        let proof: Vec<u8> = client_secret.iter().zip(&signature).map(|(a, b)| a ^ b).collect();
        let server_secret = mac(&salted, b"Server Key");
        self.expected = Some(mac(&server_secret, message.as_bytes()));
        Ok(format!("{without_proof},p={}", encode(&proof)))
    }

    /// Checks the data of `AuthenticationSASLFinal`.
    pub(crate) fn check(&self, server_final: &str) -> Result<(), String> {
        let given = server_final.strip_prefix("v=").map(decode).transpose()?;
        if given.is_some() && given == self.expected {
            Ok(())
        } else {
            Err(format!("the server signature is not valid: {server_final:?}"))
        }
    }
}

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Base64 with padding, as SCRAM uses it.
fn encode(bytes: &[u8]) -> String {
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [chunk[0], chunk.get(1).copied().unwrap_or(0), chunk.get(2).copied().unwrap_or(0)];
        let n = u32::from(b[0]) << 16 | u32::from(b[1]) << 8 | u32::from(b[2]);
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(char::from(ALPHABET[(n >> (18 - 6 * i) & 63) as usize]));
            } else {
                out.push('=');
            }
        }
    }
    out
}

fn decode(text: &str) -> Result<Vec<u8>, String> {
    let bad = || format!("{text:?} is not base64");
    let body = text.trim_end_matches('=');
    if !text.len().is_multiple_of(4) || text.len() - body.len() > 2 {
        return Err(bad());
    }
    let mut out = Vec::new();
    let mut n = 0u32;
    let mut bits = 0;
    for c in body.bytes() {
        let v = ALPHABET.iter().position(|&a| a == c).ok_or_else(bad)?;
        n = n << 6 | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((n >> bits & 0xff) as u8);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_both_ways() {
        for (bytes, text) in [(&b""[..], ""), (b"a", "YQ=="), (b"ab", "YWI="), (b"abc", "YWJj")] {
            assert_eq!(encode(bytes), text);
            assert_eq!(decode(text).unwrap(), bytes);
        }
        assert!(decode("Zm9").is_err());
        assert!(decode("Z!==").is_err());
    }

    /// The example of RFC 7677 section 3.
    #[test]
    fn the_example_of_rfc_7677() {
        let mut c = Client::with_part("pencil", "user", "rOprNGfwEbeRWgbNEkqO");
        assert_eq!(c.first(), "n,,n=user,r=rOprNGfwEbeRWgbNEkqO");
        let server_first = "r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096";
        assert_eq!(
            c.last(server_first).unwrap(),
            "c=biws,r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,p=dHzbZapWIk4jUhN+Ute9ytag9zjfMHgsqmmiz7AndVQ="
        );
        assert!(c.check("v=6rriTRBi23WpRR/wtup+mMhUZUn/dB5nLTJRsjl95G4=").is_ok());
        assert!(c.check("v=AAAA").is_err());
        assert!(c.check("e=invalid-proof").is_err());
        // The second time the salted password comes from the cache.
        let mut again = Client::with_part("pencil", "user", "rOprNGfwEbeRWgbNEkqO");
        assert_eq!(again.last(server_first).unwrap(), c.last(server_first).unwrap());
    }

    #[test]
    fn a_server_part_must_extend_the_client_part() {
        let mut c = Client::with_part("pencil", "", "abc");
        assert!(c.last("r=xyz123,s=AAAA,i=1").is_err());
        assert!(c.last("r=abc,s=AAAA,i=1").is_err());
        assert!(c.last("r=abc1,s=AAAA,i=0").is_err());
        assert!(c.last("r=abc1,s=AAAA,i=1").is_ok());
    }
}
