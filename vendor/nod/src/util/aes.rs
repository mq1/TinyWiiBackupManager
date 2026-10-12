use tracing::instrument;

use crate::{
    common::KeyBytes,
    disc::{
        SECTOR_SIZE,
        wii::{HASHES_SIZE, SECTOR_DATA_SIZE},
    },
    util::array_ref,
};

#[cfg(feature = "openssl")]
thread_local! {
    static ENC_CIPHER_CTX: std::cell::RefCell<openssl::cipher_ctx::CipherCtx> = {
        let cipher = openssl::cipher::Cipher::fetch(None, "AES-128-CBC", None).unwrap();
        let mut ctx = openssl::cipher_ctx::CipherCtx::new().unwrap();
        ctx.set_padding(false);
        ctx.encrypt_init(Some(&cipher), None, None).unwrap();
        std::cell::RefCell::new(ctx)
    };
    static DEC_CIPHER_CTX: std::cell::RefCell<openssl::cipher_ctx::CipherCtx> = {
        let cipher = openssl::cipher::Cipher::fetch(None, "AES-128-CBC", None).unwrap();
        let mut ctx = openssl::cipher_ctx::CipherCtx::new().unwrap();
        ctx.set_padding(false);
        ctx.decrypt_init(Some(&cipher), None, None).unwrap();
        std::cell::RefCell::new(ctx)
    };
}

/// Encrypts data in-place using AES-128-CBC with the given key and IV.
pub fn aes_cbc_encrypt(key: &KeyBytes, iv: &KeyBytes, data: &mut [u8]) {
    assert_eq!(data.len() % 16, 0);
    #[cfg(not(feature = "openssl"))]
    {
        use aes::cipher::{BlockModeEncrypt, KeyIvInit, block_padding::NoPadding};
        <cbc::Encryptor<aes::Aes128>>::new(key.into(), iv.into())
            .encrypt_padded::<NoPadding>(data, data.len())
            .unwrap();
    }
    #[cfg(feature = "openssl")]
    ENC_CIPHER_CTX.with_borrow_mut(|ctx| {
        ctx.encrypt_init(None, Some(key), Some(iv)).unwrap();
        let len = unsafe {
            // The openssl crate doesn't provide a safe API for using the same inbuf/outbuf.
            // However, this is valid with AES-CBC and no padding. Create a copy of the input
            // slice to appease the borrow checker.
            let input = std::slice::from_raw_parts(data.as_ptr(), data.len());
            ctx.cipher_update_unchecked(input, Some(data))
        }
        .unwrap();
        assert_eq!(len, data.len());
    });
}

/// Decrypts data in-place using AES-128-CBC with the given key and IV.
pub fn aes_cbc_decrypt(key: &KeyBytes, iv: &KeyBytes, data: &mut [u8]) {
    assert_eq!(data.len() % 16, 0);
    #[cfg(not(feature = "openssl"))]
    {
        use aes::cipher::{BlockModeDecrypt, KeyIvInit, block_padding::NoPadding};
        <cbc::Decryptor<aes::Aes128>>::new(key.into(), iv.into())
            .decrypt_padded::<NoPadding>(data)
            .unwrap();
    }
    #[cfg(feature = "openssl")]
    DEC_CIPHER_CTX.with_borrow_mut(|ctx| {
        ctx.decrypt_init(None, Some(key), Some(iv)).unwrap();
        let len = unsafe {
            // The openssl crate doesn't provide a safe API for using the same inbuf/outbuf.
            // However, this is valid with AES-CBC and no padding. Create a copy of the input
            // slice to appease the borrow checker.
            let input = std::slice::from_raw_parts(data.as_ptr(), data.len());
            ctx.cipher_update_unchecked(input, Some(data))
        }
        .unwrap();
        assert_eq!(len, data.len());
    });
}

/// Decrypts data buffer-to-buffer using AES-128-CBC with the given key and IV.
pub fn aes_cbc_decrypt_b2b(key: &KeyBytes, iv: &KeyBytes, data: &[u8], out: &mut [u8]) {
    assert_eq!(data.len() % 16, 0);
    assert_eq!(data.len(), out.len());
    #[cfg(not(feature = "openssl"))]
    {
        use aes::cipher::{BlockModeDecrypt, KeyIvInit, block_padding::NoPadding};
        <cbc::Decryptor<aes::Aes128>>::new(key.into(), iv.into())
            .decrypt_padded_b2b::<NoPadding>(data, out)
            .unwrap();
    }
    #[cfg(feature = "openssl")]
    DEC_CIPHER_CTX.with_borrow_mut(|ctx| {
        ctx.decrypt_init(None, Some(key), Some(iv)).unwrap();
        let len = unsafe { ctx.cipher_update_unchecked(data, Some(out)) }.unwrap();
        assert_eq!(len, out.len());
    });
}

/// Encrypts a Wii partition sector in-place.
#[instrument(skip_all)]
pub fn encrypt_sector(out: &mut [u8; SECTOR_SIZE], key: &KeyBytes) {
    aes_cbc_encrypt(key, &[0u8; 16], &mut out[..HASHES_SIZE]);
    // Data IV from encrypted hash block
    let iv = *array_ref![out, 0x3D0, 16];
    aes_cbc_encrypt(key, &iv, &mut out[HASHES_SIZE..]);
}

/// Decrypts a Wii partition sector in-place.
#[instrument(skip_all)]
pub fn decrypt_sector(out: &mut [u8; SECTOR_SIZE], key: &KeyBytes) {
    // Data IV from encrypted hash block
    let iv = *array_ref![out, 0x3D0, 16];
    aes_cbc_decrypt(key, &[0u8; 16], &mut out[..HASHES_SIZE]);
    aes_cbc_decrypt(key, &iv, &mut out[HASHES_SIZE..]);
}

/// Decrypts a Wii partition sector buffer-to-buffer.
#[instrument(skip_all)]
pub fn decrypt_sector_b2b(data: &[u8; SECTOR_SIZE], out: &mut [u8; SECTOR_SIZE], key: &KeyBytes) {
    // Data IV from encrypted hash block
    let iv = *array_ref![data, 0x3D0, 16];
    aes_cbc_decrypt_b2b(key, &[0u8; 16], &data[..HASHES_SIZE], &mut out[..HASHES_SIZE]);
    aes_cbc_decrypt_b2b(key, &iv, &data[HASHES_SIZE..], &mut out[HASHES_SIZE..]);
}

/// Decrypts a Wii partition sector data (excluding hashes) buffer-to-buffer.
#[instrument(skip_all)]
pub fn decrypt_sector_data_b2b(
    data: &[u8; SECTOR_SIZE],
    out: &mut [u8; SECTOR_DATA_SIZE],
    key: &KeyBytes,
) {
    // Data IV from encrypted hash block
    let iv = *array_ref![data, 0x3D0, 16];
    aes_cbc_decrypt_b2b(key, &iv, &data[HASHES_SIZE..], out);
}
