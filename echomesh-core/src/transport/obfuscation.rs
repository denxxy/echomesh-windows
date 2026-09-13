use bytes::{BufMut, BytesMut};
use rand::RngCore;

pub const TLS_HANDSHAKE_CONTENT_TYPE: u8 = 0x16;
pub const TLS_CLIENT_HELLO_HANDSHAKE_TYPE: u8 = 0x01;
pub const TLS_LEGACY_RECORD_VERSION: u16 = 0x0301;
pub const TLS_LEGACY_CLIENT_VERSION: u16 = 0x0303;
pub const TLS_1_3_VERSION: u16 = 0x0304;
pub const TLS_EXT_SERVER_NAME: u16 = 0x0000;
pub const TLS_EXT_SUPPORTED_GROUPS: u16 = 0x000a;
pub const TLS_EXT_SIGNATURE_ALGORITHMS: u16 = 0x000d;
pub const TLS_EXT_SUPPORTED_VERSIONS: u16 = 0x002b;
pub const TLS_EXT_KEY_SHARE: u16 = 0x0033;
pub const TLS_AES_128_GCM_SHA256: u16 = 0x1301;
pub const TLS_AES_256_GCM_SHA384: u16 = 0x1302;
pub const TLS_CHACHA20_POLY1305_SHA256: u16 = 0x1303;
pub const DEFAULT_SECRET_TOKEN: &[u8] = b"";

pub struct PseudoTlsBuilder { secret_token: Vec<u8>, sni_host: String, token_in_sni: bool, cipher_suites: Vec<u16> }
impl PseudoTlsBuilder {
    pub fn new(secret_token: impl Into<Vec<u8>>, sni_host: impl Into<String>) -> Self { Self { secret_token: secret_token.into(), sni_host: sni_host.into(), token_in_sni: false, cipher_suites: vec![TLS_AES_128_GCM_SHA256,TLS_AES_256_GCM_SHA384,TLS_CHACHA20_POLY1305_SHA256] } }
    pub fn embed_in_sni(mut self, enabled: bool) -> Self { self.token_in_sni=enabled; self }
    pub fn build(&self)->Vec<u8>{
        let mut random=[0u8;32];rand::thread_rng().fill_bytes(&mut random);if !self.token_in_sni&&!self.secret_token.is_empty(){let n=self.secret_token.len().min(32);random[..n].copy_from_slice(&self.secret_token[..n]);}
        let sni=if self.token_in_sni&&!self.secret_token.is_empty(){format!("{}.{}",hex::encode(&self.secret_token),self.sni_host)}else{self.sni_host.clone()};
        let mut hs=BytesMut::with_capacity(512);hs.put_u16(TLS_LEGACY_CLIENT_VERSION);hs.put_slice(&random);hs.put_u8(32);hs.put_bytes(0x22,32);hs.put_u16((self.cipher_suites.len()*2)as u16);for cs in &self.cipher_suites{hs.put_u16(*cs);}hs.put_u8(1);hs.put_u8(0);
        let mut ex=BytesMut::with_capacity(256);let sb=sni.as_bytes();let sl=1+2+sb.len();ex.put_u16(TLS_EXT_SERVER_NAME);ex.put_u16((2+sl)as u16);ex.put_u16(sl as u16);ex.put_u8(0);ex.put_u16(sb.len()as u16);ex.put_slice(sb);
        ex.put_u16(TLS_EXT_SUPPORTED_VERSIONS);ex.put_u16(5);ex.put_u8(4);ex.put_u16(TLS_1_3_VERSION);ex.put_u16(TLS_LEGACY_CLIENT_VERSION);
        ex.put_u16(TLS_EXT_SUPPORTED_GROUPS);ex.put_u16(6);ex.put_u16(4);ex.put_u16(0x001d);ex.put_u16(0x0017);
        ex.put_u16(TLS_EXT_SIGNATURE_ALGORITHMS);ex.put_u16(8);ex.put_u16(6);ex.put_u16(0x0403);ex.put_u16(0x0804);ex.put_u16(0x0807);
        ex.put_u16(TLS_EXT_KEY_SHARE);ex.put_u16(38);ex.put_u16(36);ex.put_u16(0x001d);ex.put_u16(32);let mut ks=[0u8;32];rand::thread_rng().fill_bytes(&mut ks);ex.put_slice(&ks);
        hs.put_u16(ex.len()as u16);hs.put_slice(&ex);let hl=hs.len();let total=4+hl;let mut r=Vec::with_capacity(5+total);r.push(TLS_HANDSHAKE_CONTENT_TYPE);r.extend_from_slice(&TLS_LEGACY_RECORD_VERSION.to_be_bytes());r.extend_from_slice(&(total as u16).to_be_bytes());r.push(TLS_CLIENT_HELLO_HANDSHAKE_TYPE);r.push(((hl>>16)&0xff)as u8);r.push(((hl>>8)&0xff)as u8);r.push((hl&0xff)as u8);r.extend_from_slice(&hs);r
    }
}
#[cfg(test)]mod tests{use super::*;#[test]fn embeds_explicit_token_in_random(){let t=b"abcdef0123456789abcdef0123456789";let b=PseudoTlsBuilder::new(t.to_vec(),"cloudflare.com").build();assert_eq!(&b[11..43],&t[..32]);}#[test]fn empty_token_is_not_a_shared_default(){assert!(DEFAULT_SECRET_TOKEN.is_empty());let a=PseudoTlsBuilder::new(Vec::<u8>::new(),"cloudflare.com").build();let b=PseudoTlsBuilder::new(Vec::<u8>::new(),"cloudflare.com").build();assert_ne!(&a[11..43],&b[11..43]);}}
