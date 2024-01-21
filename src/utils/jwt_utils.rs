use chrono::Utc;
use dotenv::dotenv;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::env;
use std::marker::PhantomData;

use crate::error::resp_errors::RespErrors;

use super::crypto_utils::{decrypt_payload, encrypt_payload};

#[derive(Serialize, Deserialize)]
pub struct ClaimsToken<'a, T> {
    pub payload: T,
    pub iat: i64,
    pub exp: i64,
    lifetime: PhantomData<&'a ()>,
}

#[derive(Serialize, Deserialize)]
pub struct EncryptedToken {
    pub order: [u8; 16],
    pub content: Vec<u8>,
    pub iat: i64,
    pub exp: i64,
}

pub fn create_token<T>(payload: &T, exp_at: i64) -> String
where
    T: Serialize + Deserialize<'static>,
{
    dotenv().ok();
    let secret_key = env::var("TOKEN_SECRET").expect("TOKEN_SECRET must be set");
    let encoding_key = EncodingKey::from_secret(secret_key.as_ref());
    let claims = ClaimsToken {
        payload: payload.to_owned(),
        iat: Utc::now().timestamp(),
        exp: exp_at,
        lifetime: PhantomData,
    };
    let encrypted = encrypt_payload(&claims).expect("Failed to encrypt token");
    let encrypted_token = EncryptedToken {
        order: encrypted.order,
        content: encrypted.content,
        iat: claims.iat,
        exp: claims.exp,
    };
    match encode(&Header::default(), &encrypted_token, &encoding_key) {
        Ok(token) => token,
        Err(_) => {
            return String::from("");
        }
    }
}

pub fn decode_token<T: for<'a> DeserializeOwned>(
    token: &str,
) -> Result<ClaimsToken<T>, RespErrors<String>> {
    dotenv().ok();
    let secret_key = env::var("TOKEN_SECRET").expect("TOKEN_SECRET must be set");
    let decoding_key = DecodingKey::from_secret(secret_key.as_ref());
    let validation = Validation::new(Algorithm::HS256);
    let token_decrypt = decode::<EncryptedToken>(token, &decoding_key, &validation);

    match token_decrypt {
        Ok(token) => {
            let iv = token.claims.order;
            let cipher_text = token.claims.content;

            match decrypt_payload::<ClaimsToken<T>>(&iv, &cipher_text) {
                Ok(payload) => {
                    if payload.exp < Utc::now().timestamp() {
                        Err(RespErrors::new("Token", "Expired", None))
                    } else {
                        Ok(payload)
                    }
                }
                Err(_) => {
                    return Err(RespErrors::new("Token", "Expired", None));
                }
            }
        }
        Err(_) => {
            return Err(RespErrors::new("Token", "Expired", None));
        }
    }
}
